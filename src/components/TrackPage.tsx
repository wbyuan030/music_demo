import { usePlayerStore } from "../store/Player"

export default function TrackPage() {
  const currentTrack = usePlayerStore((state) => state.currentTrack)
  return (
    <img src={currentTrack?.coverUrl}></img>
  )
}
