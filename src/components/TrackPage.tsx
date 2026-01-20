import { usePlayerStore } from "../store/Player"




export default function TrackPage() {
  const currentTrack = usePlayerStore((state) => state.currentTrack)
  if (currentTrack == null) {
    return (
      <div className="bg-neutral-900">
        <h1 className="font-semibold">No Track is Selected</h1>
      </div>
    )
  }
  return (
    <div className="bg-neutral-900">
      <img className="object-cover items-center justify-center" src={currentTrack?.coverUrl} referrerPolicy="no-referrer" />
      <h3>{currentTrack?.title}</h3>
      <h4>{currentTrack?.artist}</h4>
    </div>
  )
}
