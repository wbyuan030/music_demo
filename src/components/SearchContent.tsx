import { Loader } from "lucide-react"
import { usePlayerStore } from "../store/Player"
import { useSearchStore } from "../store/Search"
import { TrackCard } from "./TrackCard"

export default function SearchContent() {

  const tracks = useSearchStore((state) => state.tracks)
  const loadingState = useSearchStore((state) => state.isLoading)
  const setTrack = usePlayerStore((state) => state.setCurrentTrack)
  if (loadingState) {
    return (<LoadingPage />)
  }
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3 p-6 w-full">
      {
        tracks.length === 0 ? (
          <div className="col-span-full py-10 text-center text-sm text-neutral-500 border border-dashed border-neutral-800 rounded-xl bg-neutral-900/30">
            没有找到结果，换个关键词试试
          </div>
        ) : (
          tracks.map((track) => (
            <TrackCard key={track.id} track={track} onClick={setTrack} />
          ))
        )
      }
    </div>
  )
}


const LoadingPage = () => {
  return (
    <div className="items-center justify-center flex flex-1 w-full h-full">
      <div className="flex flex-col items-center gap-3">
        <Loader className="size-8 animate-spin text-emerald-400" />
        <span className="text-sm text-neutral-400 font-medium">搜索中...</span>
      </div>
    </div>
  )
}
