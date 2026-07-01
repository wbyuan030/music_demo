import { create } from 'zustand'
import type { PlayerState } from '../types/player'
import type { Track } from '../types/track';
import { safeInvoke } from '../services/invoke';

export const usePlayerStore = create<PlayerState>((set, get) => ({
  currentTrack: null,
  isPlaying: false,
  currentTime: 0,
  isLoading: false,
  isLiked: false,
  onTogglePlay: async function () {
    const { currentTrack, isPlaying } = get()
    if (currentTrack == null) return
    if (isPlaying) {
      await safeInvoke("handle_event", { event: JSON.stringify({ action: "pause" }) })
      set(() => ({ isPlaying: false }))
    } else {
      await safeInvoke("handle_event", { event: JSON.stringify({ action: "recovery" }) })
      set(() => ({ isLoading: true }))
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
    set(() => ({ currentTrack: null }))
  },
  setCurrentTrack: async function (track: Track) {
    set(() => ({ currentTime: 0 }))
    set(() => ({ currentTrack: track }))
    set(() => ({ isLoading: true }))
    await safeInvoke("handle_event", { event: JSON.stringify({ action: "play", id: track.id }) })
    set(() => ({ isLoading: false }))
    set(() => ({ isPlaying: true }))
    set(() => ({ currentTime: 0 }))
  },
  setProgress: async function (t: number) {
    set(() => ({ currentTime: t }))
  }
}))
