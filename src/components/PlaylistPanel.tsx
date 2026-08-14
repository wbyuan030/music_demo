import { useEffect, useMemo, useState, type FormEvent } from "react"
import type { Track } from "../types/track"
import { usePlaylistStore } from "../store/Playlist"

export interface PlaylistPanelProps {
  availableTracks?: Track[]
}

export default function PlaylistPanel({ availableTracks = [] }: PlaylistPanelProps) {
  const playlists = usePlaylistStore((state) => state.playlists)
  const isLoading = usePlaylistStore((state) => state.isLoading)
  const loadPlaylists = usePlaylistStore((state) => state.loadPlaylists)
  const createPlaylist = usePlaylistStore((state) => state.createPlaylist)
  const renamePlaylist = usePlaylistStore((state) => state.renamePlaylist)
  const deletePlaylist = usePlaylistStore((state) => state.deletePlaylist)
  const addTrack = usePlaylistStore((state) => state.addTrack)
  const reorderTrack = usePlaylistStore((state) => state.reorderTrack)
  const removeTrack = usePlaylistStore((state) => state.removeTrack)
  const playPlaylist = usePlaylistStore((state) => state.playPlaylist)
  const [selectedId, setSelectedId] = useState<string>()
  const [newName, setNewName] = useState("")
  const [renameValue, setRenameValue] = useState<string>()
  const [trackToAdd, setTrackToAdd] = useState("")

  useEffect(() => {
    void loadPlaylists()
  }, [loadPlaylists])

  const selected = useMemo(
    () => playlists.find((playlist) => playlist.id === selectedId) ?? playlists[0],
    [playlists, selectedId],
  )
  const availableToAdd = availableTracks.filter(
    (track) => !selected?.tracks.some((item) => item.id === track.id),
  )


  async function handleCreate(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const playlist = await createPlaylist(newName)
    if (playlist) {
      setNewName("")
      setSelectedId(playlist.id)
    }
  }

  async function handleRename(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    if (selected) await renamePlaylist(selected.id, renameValue ?? selected.name)
  }

  async function handleAddTrack(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    if (selected && trackToAdd) {
      const updated = await addTrack(selected.id, trackToAdd)
      if (updated) setTrackToAdd("")
    }
  }

  async function handleDelete() {
    if (!selected) return
    const deleted = await deletePlaylist(selected.id)
    if (deleted) setSelectedId(undefined)
  }

  return (
    <section className="flex flex-col gap-4 rounded-xl border border-neutral-800 bg-neutral-900/50 p-4 text-neutral-100">
      <div className="flex items-center justify-between gap-3">
        <h2 className="text-lg font-semibold">Playlists</h2>
        {isLoading && <span className="text-xs text-neutral-500">Loading...</span>}
      </div>

      <form className="flex gap-2" onSubmit={handleCreate}>
        <input
          value={newName}
          onChange={(event) => setNewName(event.target.value)}
          placeholder="New playlist name"
          className="min-w-0 flex-1 rounded-md border border-neutral-700 bg-neutral-950 px-3 py-2 text-sm outline-none focus:border-emerald-500"
        />
        <button type="submit" className="rounded-md bg-emerald-500 px-3 py-2 text-sm font-medium text-neutral-950">
          Create
        </button>
      </form>

      {playlists.length === 0 ? (
        <p className="py-6 text-center text-sm text-neutral-500">No playlists yet.</p>
      ) : (
        <div className="grid gap-4 md:grid-cols-[minmax(10rem,15rem)_1fr]">
          <div className="flex flex-col gap-1">
            {playlists.map((playlist) => (
              <button
                type="button"
                key={playlist.id}
                onClick={() => {
                  setSelectedId(playlist.id)
                  setRenameValue(undefined)
                }}
                className={`rounded-md px-3 py-2 text-left text-sm ${selected?.id === playlist.id ? "bg-emerald-500/20 text-emerald-300" : "text-neutral-300 hover:bg-neutral-800"}`}
              >
                <span className="block truncate">{playlist.name}</span>
                <span className="text-xs text-neutral-500">{playlist.trackCount} tracks</span>
              </button>
            ))}
          </div>

          {selected && (
            <div className="flex min-w-0 flex-col gap-3">
              <div className="flex flex-wrap items-center gap-2">
                <form className="flex min-w-0 flex-1 gap-2" onSubmit={handleRename}>
                  <input
                    value={renameValue ?? selected.name}
                    onChange={(event) => setRenameValue(event.target.value)}
                    className="min-w-0 flex-1 rounded-md border border-neutral-700 bg-neutral-950 px-3 py-2 text-sm outline-none focus:border-emerald-500"
                    aria-label="Playlist name"
                  />
                  <button type="submit" className="rounded-md border border-neutral-700 px-3 py-2 text-sm hover:bg-neutral-800">
                    Rename
                  </button>
                </form>
                <button type="button" onClick={() => void handleDelete()} className="rounded-md border border-red-900 px-3 py-2 text-sm text-red-300 hover:bg-red-950/40">
                  Delete
                </button>
              </div>

              {availableToAdd.length > 0 && (
                <form className="flex gap-2" onSubmit={handleAddTrack}>
                  <select
                    value={trackToAdd}
                    onChange={(event) => setTrackToAdd(event.target.value)}
                    className="min-w-0 flex-1 rounded-md border border-neutral-700 bg-neutral-950 px-3 py-2 text-sm"
                    aria-label="Track to add"
                  >
                    <option value="">Add a track...</option>
                    {availableToAdd.map((track) => (
                      <option key={track.id} value={track.id}>{track.title} — {track.artist}</option>
                    ))}
                  </select>
                  <button type="submit" className="rounded-md bg-neutral-800 px-3 py-2 text-sm hover:bg-neutral-700">
                    Add
                  </button>
                </form>
              )}

              <div className="flex flex-col gap-1">
                {selected.tracks.map((track, index) => (
                  <div key={track.id} className="flex items-center gap-2 rounded-md px-2 py-2 hover:bg-neutral-800">
                    <button type="button" onClick={() => playPlaylist(selected.id, track.id)} className="min-w-0 flex-1 truncate text-left text-sm">
                      <span className="text-neutral-100">{track.title}</span>
                      <span className="ml-2 text-neutral-500">{track.artist}</span>
                    </button>
                    <div className="flex shrink-0 items-center">
                      <button
                        type="button"
                        disabled={index === 0}
                        onClick={() => void reorderTrack(selected.id, track.id, index - 1)}
                        className="px-1 text-xs text-neutral-500 hover:text-emerald-300 disabled:cursor-not-allowed disabled:opacity-30"
                        aria-label={`Move ${track.title} up`}
                      >
                        ↑
                      </button>
                      <button
                        type="button"
                        disabled={index === selected.tracks.length - 1}
                        onClick={() => void reorderTrack(selected.id, track.id, index + 1)}
                        className="px-1 text-xs text-neutral-500 hover:text-emerald-300 disabled:cursor-not-allowed disabled:opacity-30"
                        aria-label={`Move ${track.title} down`}
                      >
                        ↓
                      </button>
                    </div>
                    <button type="button" onClick={() => void removeTrack(selected.id, track.id)} className="px-2 text-xs text-neutral-500 hover:text-red-300">
                      Remove
                    </button>
                  </div>
                ))}
                {selected.tracks.length === 0 && <p className="py-4 text-sm text-neutral-500">This playlist is empty.</p>}
              </div>
            </div>
          )}
        </div>
      )}
    </section>
  )
}
