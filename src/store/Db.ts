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

interface LikedPlays {
  likedTracks: Track[]
  getLikedTracks: () => void
}

export const useLikedStore = create<LikedPlays>((set) => ({
  likedTracks: [],
  getLikedTracks: async function () {
    const trackList = await invoke<Track[]>("get_liked_tracks")
    set(() => ({ likedTracks: trackList }))
  },
}))
