import { create } from "zustand";
import type { Track } from "../types/track";
import { safeInvoke } from "../services/invoke";

interface RecentPlays {
  recentTracks: Track[]
  getRecentTracks: () => void
}
export const useRecentStore = create<RecentPlays>((set) => ({
  recentTracks: [],
  getRecentTracks: async function () {
    const trackList = await safeInvoke<Track[]>("list_recent_tracks")
    if (trackList) set(() => ({ recentTracks: trackList }))
  },
}))

interface LikedPlays {
  likedTracks: Track[]
  getLikedTracks: () => void
}

export const useLikedStore = create<LikedPlays>((set) => ({
  likedTracks: [],
  getLikedTracks: async function () {
    const trackList = await safeInvoke<Track[]>("get_liked_tracks")
    if (trackList) set(() => ({ likedTracks: trackList }))
  },
}))
