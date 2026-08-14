import { create } from 'zustand'
import type { PlayerState, PlaybackStateView } from '../types/player'
import type { Track } from '../types/track'
import { safeInvoke } from '../services/invoke'
import { useErrorStore } from './Error'
import { useLikedStore } from './Db'

/** 加载兜底：超过该时长仍未收到 play_start/play_failed 视为失败。 */
const LOAD_TIMEOUT_MS = 30_000
const MAX_VOLUME = 50

let loadTimeout: number | undefined

function clearLoadTimeout() {
  if (loadTimeout !== undefined) {
    window.clearTimeout(loadTimeout)
    loadTimeout = undefined
  }
}

export const usePlayerStore = create<PlayerState>((set, get) => ({
  currentTrack: null,
  isPlaying: false,
  currentTime: 0,
  isLoading: false,
  isEnded: false,
  isLiked: false,
  volume: MAX_VOLUME,
  lastVolume: MAX_VOLUME,
  isMuted: false,
  onTogglePlay: async function () {
    const { currentTrack, isPlaying, isLoading, isEnded } = get()
    if (currentTrack == null || isLoading) return

    if (isEnded) {
      // 自然结束或失败后 sink 已不可恢复，必须重新走 play/load 流程。
      void get().setCurrentTrack(currentTrack)
      return
    }

    const action = isPlaying ? 'pause' : 'recovery'
    const trackId = currentTrack.id
    const result = await safeInvoke<void>('handle_event', {
      event: JSON.stringify({ action }),
    })
    if (result === null || get().currentTrack?.id !== trackId) return
    set(() => ({ isPlaying: !isPlaying }))
  },

  onToggleLike: async () => {
    const currentTrack = get().currentTrack
    if (currentTrack == null) return

    const trackId = currentTrack.id
    const result = await safeInvoke<void>('toggle_liked_track', { id: trackId })
    if (result === null) return

    if (get().currentTrack?.id === trackId) {
      set((state) => ({ isLiked: !state.isLiked }))
    }
    const refreshed = await useLikedStore.getState().getLikedTracks()
    if (refreshed) {
      const latest = useLikedStore.getState().likedTracks
      get().syncLikedState(trackId, latest.some((track) => track.id === trackId))
    }
  },

  onSeek: async function (time: number) {
    const { currentTrack, currentTime: previousTime } = get()
    if (
      currentTrack == null ||
      !Number.isFinite(time) ||
      time < 0 ||
      time > currentTrack.duration
    ) {
      console.error('time setting over range', {
        totalTime: currentTrack?.duration,
        time,
      })
      return
    }

    set(() => ({ currentTime: time }))
    const result = await safeInvoke<void>('handle_event', {
      event: JSON.stringify({ action: 'seek', time }),
    })
    if (result === null && get().currentTrack?.id === currentTrack.id) {
      set(() => ({ currentTime: previousTime }))
    }
  },

  onVolumeChange: async function (requestedVolume: number) {
    if (!Number.isFinite(requestedVolume)) return
    const volume = Math.min(MAX_VOLUME, Math.max(0, requestedVolume))
    const result = await safeInvoke<void>('handle_event', {
      event: JSON.stringify({ action: 'volume', volume }),
    })
    if (result === null) return

    set((state) => ({
      volume,
      isMuted: volume === 0,
      lastVolume: volume > 0 ? volume : state.lastVolume,
    }))
  },

  onToggleMute: () => {
    const { isMuted, volume, lastVolume } = get()
    void get().onVolumeChange(isMuted || volume === 0 ? lastVolume : 0)
  },

  clearCurrentTrack: () => {
    clearLoadTimeout()
    set(() => ({
      currentTrack: null,
      isPlaying: false,
      isLoading: false,
      isEnded: false,
      currentTime: 0,
      isLiked: false,
    }))
  },

  setCurrentTrack: async function (track: Track) {
    const isLiked = useLikedStore.getState().likedTracks.some((item) => item.id === track.id)
    // 只进入加载态；真正的播放中由 play_start 事件驱动。
    set(() => ({
      currentTrack: track,
      currentTime: 0,
      isPlaying: false,
      isLoading: true,
      isEnded: false,
      isLiked,
    }))
    clearLoadTimeout()
    loadTimeout = window.setTimeout(() => {
      get().onPlaybackFailed('加载超时')
    }, LOAD_TIMEOUT_MS)

    const result = await safeInvoke<void>('handle_event', {
      event: JSON.stringify({ action: 'play', id: track.id }),
    })
    if (result === null && get().currentTrack?.id === track.id) {
      clearLoadTimeout()
      set(() => ({ isLoading: false, isEnded: true }))
    }
  },

  setProgress: (t: number) => {
    set(() => ({ currentTime: t }))
  },

  syncLikedState: (trackId: string, liked: boolean) => {
    if (get().currentTrack?.id !== trackId) return
    set(() => ({ isLiked: liked }))
  },

  onPlaybackStarted: () => {
    clearLoadTimeout()
    set(() => ({ isPlaying: true, isLoading: false, isEnded: false }))
  },

  onPlaybackEnded: () => {
    clearLoadTimeout()
    set(() => ({ isPlaying: false, isLoading: false, isEnded: true }))
  },

  onPlaybackFailed: (message: string) => {
    clearLoadTimeout()
    set(() => ({ isPlaying: false, isLoading: false, isEnded: true }))
    useErrorStore.getState().pushError(`播放失败: ${message}`)
  },

  onBackendState: (snapshot: PlaybackStateView) => {
    // 挂载/重连对账：用后端快照纠正本地状态（事件可能因监听时机丢失）。
    const { currentTrack } = get()
    if (snapshot.trackId && currentTrack && snapshot.trackId !== currentTrack.id) return
    clearLoadTimeout()
    set((state) => {
      const synced = snapshot.phase === 'playing' || snapshot.phase === 'paused'
      return {
        isLoading: snapshot.phase === 'loading',
        isPlaying: snapshot.phase === 'playing',
        isEnded: snapshot.phase === 'idle' && snapshot.error != null
          ? true
          : synced
            ? false
            : state.isEnded,
        ...(synced ? { currentTime: snapshot.positionSecs } : {}),
      }
    })
    if (snapshot.phase === 'idle' && snapshot.error && currentTrack) {
      useErrorStore.getState().pushError(`播放失败: ${snapshot.error}`)
    }
  },
}))
