import { create } from 'zustand'
import type { PlayerState } from '../types/player'
import type { Track } from '../types/track';
import { safeInvoke } from '../services/invoke';
import { useErrorStore } from './Error';

/** 加载兜底：超过该时长仍未收到 play_start/play_failed 视为失败。 */
const LOAD_TIMEOUT_MS = 30_000;

let loadTimeout: number | undefined;

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
  isLiked: false,
  onTogglePlay: async function () {
    const { currentTrack, isPlaying, isLoading } = get()
    if (currentTrack == null || isLoading) return
    if (isPlaying) {
      await safeInvoke("handle_event", { event: JSON.stringify({ action: "pause" }) })
      set(() => ({ isPlaying: false }))
    } else {
      // sink 已加载，恢复立即生效，无需 loading 态
      await safeInvoke("handle_event", { event: JSON.stringify({ action: "recovery" }) })
      set(() => ({ isPlaying: true }))
    }
  },
  onToggleLike: async () => {
    const currentTrack = get().currentTrack
    const ok = await safeInvoke("toggle_liked_track", { id: currentTrack?.id })
    if (ok !== null) set((state) => ({ isLiked: !state.isLiked }))
  },
  onNext: () => { },
  onPrev: () => { },
  onSeek: async function (time: number) {
    const total_time = get().currentTrack?.duration;
    if (total_time == null || total_time < time) {
      console.error("time setting over range. total_time:{},setting time:{}", total_time, time)
      return
    }
    set(() => ({ currentTime: time }))
    await safeInvoke("handle_event", { event: JSON.stringify({ action: "seek", time: time }) })
  },
  clearCurrentTrack: () => {
    clearLoadTimeout()
    set(() => ({ currentTrack: null, isPlaying: false, isLoading: false, currentTime: 0 }))
  },
  setCurrentTrack: async function (track: Track) {
    // 只进入加载态；真正的"播放中"由 play_start 事件驱动。
    set(() => ({ currentTrack: track, currentTime: 0, isPlaying: false, isLoading: true }))
    clearLoadTimeout()
    loadTimeout = window.setTimeout(() => {
      get().onPlaybackFailed("加载超时")
    }, LOAD_TIMEOUT_MS)
    await safeInvoke("handle_event", { event: JSON.stringify({ action: "play", id: track.id }) })
  },
  setProgress: async function (t: number) {
    set(() => ({ currentTime: t }))
  },
  onPlaybackStarted: () => {
    clearLoadTimeout()
    set(() => ({ isPlaying: true, isLoading: false }))
  },
  onPlaybackEnded: () => {
    clearLoadTimeout()
    set(() => ({ isPlaying: false, isLoading: false }))
  },
  onPlaybackFailed: (message: string) => {
    clearLoadTimeout()
    set(() => ({ isPlaying: false, isLoading: false }))
    useErrorStore.getState().pushError(`播放失败: ${message}`)
  },
  onBackendState: (snapshot) => {
    // 挂载/重连对账：用后端快照纠正本地状态（事件可能因监听时机丢失）。
    const { currentTrack } = get()
    if (snapshot.trackId && currentTrack && snapshot.trackId !== currentTrack.id) return
    clearLoadTimeout()
    set(() => {
      const synced = snapshot.phase === 'playing' || snapshot.phase === 'paused'
      return {
        isLoading: snapshot.phase === 'loading',
        isPlaying: snapshot.phase === 'playing',
        ...(synced ? { currentTime: snapshot.positionSecs } : {}),
      }
    })
    if (snapshot.phase === 'idle' && snapshot.error && currentTrack) {
      useErrorStore.getState().pushError(`播放失败: ${snapshot.error}`)
    }
  },
}))
