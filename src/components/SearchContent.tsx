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
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 bg-black min-h-full min-w-full">
      {
        tracks.map((track) => (
          <TrackCard key={track.id} track={track} onClick={setTrack} />
        ))
      }
    </div >
  )
}


const LoadingPage = () => {
  return (
    <div className="items-center justify-center flex flex-1 w-full h-full" >
      <button type="button" className="flex  items-center justify-center w-64 h-32 !bg-indigo-500 border-blue-800 border-4 rounded-3xl" >
        <Loader className="size-10 animate-spin text-blue-500" />
        <span className="font-bold text-xl text-white"> Loading... </span>
      </button>
    </div>
  )
}
