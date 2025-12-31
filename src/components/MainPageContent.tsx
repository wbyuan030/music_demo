import { useEffect } from "react"
import { useRecentStore } from "../store/Db"
import { usePlayerStore } from "../store/Player"
import { formatTime } from "../types/track"

export default function MainPageContent() {
  const recentTracks = useRecentStore((state) => state.recentTracks)
  const setRecentTracks = useRecentStore((state) => state.getRecentTracks)
  const setTrack = usePlayerStore((state) => state.setCurrentTrack)
  useEffect(() => {
    setRecentTracks()
  }, [])
  return (
    <div className="flex flex-col items-center justify-center gap-4">
      <h4 className="text-3xl font-bold">Recent Tracks</h4>
      {
        recentTracks.map((track) => (
          <div
            key={track.id}
            className="flex flex-1 w-screen items-center gap-4 p-3 pl-6 rounded-lg hover:bg-white/10 hover:scale-105  transition-all cursor-pointer group border-b border-gray-50 last:border-0"
            onClick={() => {
              setTrack(track)
            }}
          >
            <div className="w-12 h-12 bg-gray-200 rounded shrink-0">
              {<img src={track.coverUrl} referrerPolicy="no-referrer" className="w-full rounded-lg h-full object-cover" />}
            </div>
            <div className="flex-1 min-w-0 flex flex-col justify-center gap-1">
              <h4 className="font-medium text-gray-200 text-sm group-hover:text-white truncate [&>em]:text-pink-500 [&>em]:italic"
                dangerouslySetInnerHTML={{ __html: track.title }}
              />
              <span className="flex flex-row justify-center gap-3">
                <p className="text-xs text-gray-500 truncate group-hover:text-gray-400">{track.artist}</p>
                <p className="text-xs text-gray-600 font-mono group-hover:text-gray-400">{formatTime(track.duration)}</p>
              </span>
            </div>
          </div>

        ))
      }
    </div>
  )
}
