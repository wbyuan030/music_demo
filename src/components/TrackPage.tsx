import { usePlayerStore } from "../store/Player"

function TrackContent() {
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

function TrackLayout({ TrackTopBar, TrackPlayBar, TrackContent }: any) {
  return (
    <div className="flex flex-row h-screen w-screen">
      <div className="flex h-1/5">
        <TrackTopBar />
      </div>
      <div className="flex flex-1">
        <TrackContent />
      </div>
      <div className="flex h-1/5">
        <TrackPlayBar />
      </div>
    </div>
  )
}

function TrackTopBar() {

}

function TrackPlayBar() {

}



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
