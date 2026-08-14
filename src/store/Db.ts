import { create } from "zustand";
import type { Track } from "../types/track";
import { safeInvoke } from "../services/invoke";

interface RecentPlays {
  recentTracks: Track[];
  isLoaded: boolean;
  getRecentTracks: () => Promise<boolean>;
}

export const useRecentStore = create<RecentPlays>((set) => ({
  recentTracks: [],
  isLoaded: false,
  getRecentTracks: async function () {
    const trackList = await safeInvoke<Track[]>("list_recent_tracks");
    if (trackList === null) return false;
    set(() => ({ recentTracks: trackList, isLoaded: true }));
    return true;
  },
}));

interface LikedPlays {
  likedTracks: Track[];
  isLoaded: boolean;
  getLikedTracks: () => Promise<boolean>;
}

export const useLikedStore = create<LikedPlays>((set) => ({
  likedTracks: [],
  isLoaded: false,
  getLikedTracks: async function () {
    const trackList = await safeInvoke<Track[]>("list_liked_tracks");
    if (trackList === null) return false;
    set(() => ({ likedTracks: trackList, isLoaded: true }));
    return true;
  },
}));
