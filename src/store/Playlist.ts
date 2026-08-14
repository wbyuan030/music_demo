import { create } from "zustand"
import { safeInvoke } from "../services/invoke"
import type { Track } from "../types/track"
import { useQueueStore } from "./Queue"

export interface Playlist {
  id: string
  name: string
  trackCount: number
  tracks: Track[]
}

interface PlaylistState {
  playlists: Playlist[]
  isLoaded: boolean
  isLoading: boolean
  loadPlaylists: () => Promise<boolean>
  createPlaylist: (name: string) => Promise<Playlist | null>
  renamePlaylist: (id: string, name: string) => Promise<Playlist | null>
  deletePlaylist: (id: string) => Promise<boolean>
  addTrack: (playlistId: string, trackId: string) => Promise<Playlist | null>
  reorderTrack: (playlistId: string, trackId: string, position: number) => Promise<Playlist | null>
  removeTrack: (playlistId: string, trackId: string) => Promise<Playlist | null>
  playPlaylist: (playlistId: string, trackId?: string) => boolean
}

export const usePlaylistStore = create<PlaylistState>((set, get) => ({
  playlists: [],
  isLoaded: false,
  isLoading: false,

  loadPlaylists: async () => {
    if (get().isLoaded || get().isLoading) return get().isLoaded
    set({ isLoading: true })
    const playlists = await safeInvoke<Playlist[]>("list_playlists")
    if (playlists === null) {
      set({ isLoading: false })
      return false
    }
    set({ playlists, isLoaded: true, isLoading: false })
    return true
  },

  createPlaylist: async (name) => {
    const playlist = await safeInvoke<Playlist>("create_playlist", { name })
    if (playlist === null) return null
    set((state) => ({ playlists: [...state.playlists, playlist] }))
    return playlist
  },

  renamePlaylist: async (id, name) => {
    const playlist = await safeInvoke<Playlist>("rename_playlist", { id, name })
    if (playlist === null) return null
    set((state) => ({
      playlists: state.playlists.map((item) => (item.id === playlist.id ? playlist : item)),
    }))
    return playlist
  },

  deletePlaylist: async (id) => {
    const result = await safeInvoke<void>("delete_playlist", { id })
    if (result === null) return false
    set((state) => ({ playlists: state.playlists.filter((playlist) => playlist.id !== id) }))
    return true
  },

  addTrack: async (playlistId, trackId) => {
    const playlist = await safeInvoke<Playlist>("add_playlist_track", {
      playlistId,
      trackId,
    })
    if (playlist === null) return null
    set((state) => ({
      playlists: state.playlists.map((item) => (item.id === playlist.id ? playlist : item)),
    }))
    return playlist
  },

  reorderTrack: async (playlistId, trackId, position) => {
    const playlist = await safeInvoke<Playlist>("reorder_playlist_track", {
      playlistId,
      trackId,
      position,
    })
    if (playlist === null) return null
    set((state) => ({
      playlists: state.playlists.map((item) => (item.id === playlist.id ? playlist : item)),
    }))
    return playlist
  },

  removeTrack: async (playlistId, trackId) => {
    const playlist = await safeInvoke<Playlist>("remove_playlist_track", {
      playlistId,
      trackId,
    })
    if (playlist === null) return null
    set((state) => ({
      playlists: state.playlists.map((item) => (item.id === playlist.id ? playlist : item)),
    }))
    return playlist
  },

  playPlaylist: (playlistId, trackId) => {
    const playlist = get().playlists.find((item) => item.id === playlistId)
    if (!playlist || playlist.tracks.length === 0) return false
    const track = playlist.tracks.find((item) => item.id === trackId) ?? playlist.tracks[0]
    useQueueStore.getState().playFromList(track, playlist.tracks)
    return true
  },
}))
