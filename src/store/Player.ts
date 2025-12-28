import { create } from 'zustand'
import type { PlayerState } from '../types/player'
import type { Track } from '../types/track';
import { invoke } from '@tauri-apps/api/core';


// const audio = new Audio();
export const usePlayerStore = create<PlayerState>((set, get) => ({
  currentTrack: null,
  isPlaying: false,
  currentTime: 0,
  isLiked: false,
  onTogglePlay: async function () {
    const { currentTrack, isPlaying } = get()
    if (currentTrack == null) {
      return
    }
    if (isPlaying) {
      await invoke("handle_event", { event: JSON.stringify({ action: "pause" }) });
      set(() => ({ isPlaying: false }));
    }
    else {

      await invoke("handle_event", { event: JSON.stringify({ action: "recovery" }) });
      set(() => ({ isPlaying: true }));
    }
    if (get().currentTrack != null) {
    }
  },
  onToggleLike: () => set((state) => ({ isLiked: !state.isLiked })),
  onNext: () => { },
  onPrev: () => { },
  onSeek: async function (time: number) {
    const total_time = get().currentTrack?.duration;
    if (total_time == null || total_time < time) {
      console.error("time setting over range. total_time:{},setting time:{}", total_time, time)
      return
    }
    set(() => ({ currentTime: time }))
    await invoke("handle_event", {
      event: JSON.stringify({ action: "seek", time: time })
    });

  },
  clearCurrentTrack: () => {
    set(() => ({ currentTrack: null }))
  },
  setCurrentTrack: async function (track: Track) {

    set(() => ({ currentTime: 0 }))
    set(() => ({ currentTrack: track }));
    await invoke("handle_event", { event: JSON.stringify({ action: "play", id: track.id }) });
    set(() => ({ isPlaying: true }));
    set(() => ({ currentTime: 0 }))
  },
  setProgress: async function (t: number) {
    set(() => ({ currentTime: t }))
    return
  }

}))
