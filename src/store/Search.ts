import { create } from 'zustand'
import type { Track } from '../types/track'
import { safeInvoke } from '../services/invoke'

interface SearchState {
  tracks: Array<Track>;
  search: (query: string) => void;
  isLoading: boolean;
  /** Current search source: 'all' | 'bilibili' | 'youtube' */
  source: string;
  setSource: (source: string) => void;
}

export const useSearchStore = create<SearchState>((set, get) => ({
  tracks: [],
  isLoading: false,
  source: 'all',
  setSource: (source: string) => set(() => ({ source })),
  search: async function (query: string) {
    set(() => ({ isLoading: true }))
    const source = get().source
    const result = await safeInvoke<Track[]>("search_music", { keyword: query, source })
    set(() => ({ tracks: result ?? [], isLoading: false }))
  }
}))
