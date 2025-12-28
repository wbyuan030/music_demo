import { create } from 'zustand'
import type { Track } from '../types/track'
import { invoke } from '@tauri-apps/api/core'

interface SearchState {
  tracks: Array<Track>;
  search: (query: string) => void;
  isLoading: boolean;
}

export const useSearchStore = create<SearchState>((set, get) => ({
  tracks: [],
  isLoading: false,
  search: async function (query: string) {
    set(() => ({ isLoading: true }))
    const tracks = await invoke<Track[]>("search_music", { keyword: query });
    set(() => ({ isLoading: false }))
    set(() => ({ tracks }));
  }

}))
