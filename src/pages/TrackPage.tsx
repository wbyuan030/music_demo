import { usePlayerStore } from "../store/Player"
import { useStateStore } from "../store/State"
import { ChevronLeft } from "lucide-react"
import { StateEnum } from "../types/state"
import type { ReactNode } from "react";
import MiniPlayer from "../components/MiniPlayer";

interface TrackLayoutProps {
  TrackTopBar: ReactNode;
  TrackPlayBar: ReactNode;
  TrackContent: ReactNode;
}

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
    <div className="flex flex-col gap-4 h-full w-full bg-neutral-900 items-center justify-center">
      <img className="aspect-square scale-70 object-cover! border-4 border-green-800 !rounded-full animate-[spin_5s_linear_infinite] " src={currentTrack?.coverUrl} referrerPolicy="no-referrer" />
      <h3 className="text-gray-300 font-bold">{currentTrack?.title}</h3>
      <h4 className="text-gray-400 font-semibold">{currentTrack?.artist}</h4>
    </div >
  )
}

function TrackLayout({ TrackTopBar, TrackPlayBar, TrackContent }: TrackLayoutProps) {
  return (
    <div className="flex flex-col h-screen w-screen bg-neutral-900">
      <div className="flex h-1/12">
        {TrackTopBar}
      </div>
      <div className="flex flex-1">
        {TrackContent}
      </div>
      <div className="flex h-1/5">
        {TrackPlayBar}
      </div>
    </div>
  )
}




function TrackTopBar() {
  const setCurrentState = useStateStore((state) => state.setCurrentState)
  return (
    <div className="flex flex-1 pl-4">
      <button onClick={() => { setCurrentState(StateEnum.mainPage) }} className="text-gray-300 !bg-neutral-900"><ChevronLeft /></button>
    </div>
  )
}





export default function TrackPage() {
  return (
    <div>
      <TrackLayout TrackTopBar={<TrackTopBar />} TrackPlayBar={<MiniPlayer />} TrackContent={<TrackContent />} />
    </div>
  )
}
