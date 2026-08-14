import { useEffect, useState, type MouseEvent } from "react"
import { ListPlus } from "lucide-react"
import { usePlaylistStore } from "../store/Playlist"

interface AddToPlaylistButtonProps {
  trackId: string
}

export function AddToPlaylistButton({ trackId }: AddToPlaylistButtonProps) {
  const playlists = usePlaylistStore((state) => state.playlists)
  const isLoaded = usePlaylistStore((state) => state.isLoaded)
  const isLoading = usePlaylistStore((state) => state.isLoading)
  const loadPlaylists = usePlaylistStore((state) => state.loadPlaylists)
  const addTrack = usePlaylistStore((state) => state.addTrack)
  const [isOpen, setIsOpen] = useState(false)

  useEffect(() => {
    if (isOpen && !isLoaded && !isLoading) void loadPlaylists()
  }, [isLoaded, isLoading, isOpen, loadPlaylists])

  function stopPropagation(event: MouseEvent) {
    event.stopPropagation()
  }

  return (
    <div className="relative shrink-0 touch-manipulation" onClick={stopPropagation} onPointerDown={(event) => event.stopPropagation()}>
      <button
        type="button"
        onClick={() => setIsOpen((open) => !open)}
        className="flex size-11 items-center justify-center rounded-md text-neutral-500 opacity-100 touch-manipulation transition-opacity hover:bg-neutral-700 hover:text-emerald-300 md:size-9 md:p-2 md:opacity-0 md:group-hover:opacity-100"
        aria-label="Add to playlist"
        aria-expanded={isOpen}
      >
        <ListPlus size={16} />
      </button>
      {isOpen && (
        <div className="absolute bottom-full right-0 z-20 mb-1 min-w-44 rounded-md border border-neutral-700 bg-neutral-950 p-1 shadow-xl md:bottom-auto md:top-full md:mt-1">
          {isLoading ? (
            <p className="px-2 py-2 text-xs text-neutral-500">Loading playlists...</p>
          ) : playlists.length === 0 ? (
            <p className="px-2 py-2 text-xs text-neutral-500">No playlists yet.</p>
          ) : (
            playlists.map((playlist) => (
              <button
                type="button"
                key={playlist.id}
                onClick={() => {
                  void addTrack(playlist.id, trackId).then((updated) => {
                    if (updated) setIsOpen(false)
                  })
                }}
                className="block min-h-11 w-full truncate rounded px-2 py-1.5 text-left text-xs text-neutral-200 hover:bg-neutral-800 hover:text-emerald-300"
              >
                {playlist.name}
              </button>
            ))
          )}
        </div>
      )}
    </div>
  )
}
