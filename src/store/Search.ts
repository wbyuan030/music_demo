import { create } from 'zustand'
import type { Track } from '../types/track'
import { safeInvoke } from '../services/invoke'

interface SearchState {
  tracks: Array<Track>;
  search: (query: string) => void;
  isLoading: boolean;
}

export const useSearchStore = create<SearchState>((set) => ({
  tracks: [],
  isLoading: false,
  search: async function (query: string) {
    set(() => ({ isLoading: true }))
    const tracks = await safeInvoke<Track[]>("search_music", { keyword: query })
    if (tracks) set(() => ({ tracks }))
    set(() => ({ isLoading: false }))
  }
}))
