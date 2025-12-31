import { create } from "zustand";
import type { Track } from "../types/track";
import { invoke } from "@tauri-apps/api/core";

interface RecentPlays {
  recentTracks: Track[]
  getRecentTracks: () => void
}
export const useRecentStore = create<RecentPlays>((set) => ({
  recentTracks: [],
  getRecentTracks: async function () {
    const trackList = await invoke<Track[]>("get_recent_tracks")
    set(() => ({ recentTracks: trackList }))
  },
}))

