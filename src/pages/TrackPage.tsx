import { usePlayerStore } from "../store/Player"
import { useStateStore } from "../store/State"
import { ChevronLeft } from "lucide-react"
import { StateEnum } from "../types/state"
import type { ReactNode } from "react";
import MiniPlayer from "../components/MiniPlayer";
import { HighlightedText } from "../components/HighlightedText";

interface TrackLayoutProps {
  TrackTopBar: ReactNode;
  TrackPlayBar: ReactNode;
  TrackContent: ReactNode;
}

function TrackContent() {
  const currentTrack = usePlayerStore((state) => state.currentTrack)
  const isPlaying = usePlayerStore((state) => state.isPlaying)
  if (currentTrack == null) {
    return (
      <div className="flex-1 flex items-center justify-center bg-neutral-950">
        <p className="text-neutral-500 text-sm">没有正在播放的歌曲</p>
      </div>
    )
  }
  return (
    <div className="flex flex-col gap-5 h-full w-full items-center justify-center bg-gradient-to-b from-neutral-900 via-neutral-950 to-neutral-950 px-8">
      <div className="relative">
        <div className={`absolute -inset-6 rounded-full bg-emerald-500/10 blur-2xl transition-opacity duration-500 ${isPlaying ? 'opacity-100' : 'opacity-40'}`} />
        <img
          className={`relative w-56 h-56 object-cover rounded-full ring-4 ring-neutral-800 shadow-2xl shadow-black/60 ${isPlaying ? 'animate-[spin_5s_linear_infinite]' : ''}`}
          src={currentTrack?.coverUrl}
          referrerPolicy="no-referrer"
          onError={(e) => { (e.target as HTMLImageElement).style.visibility = 'hidden' }}
        />
      </div>
      <div className="text-center space-y-1.5">
        <h3 className="text-2xl font-bold text-white tracking-tight">
          <HighlightedText text={currentTrack.title} />
        </h3>
        <h4 className="text-neutral-400 font-medium">{currentTrack.artist}</h4>
      </div>
    </div>
  )
}

function TrackLayout({ TrackTopBar, TrackPlayBar, TrackContent }: TrackLayoutProps) {
  return (
    <div className="flex flex-col h-screen w-screen bg-neutral-950">
      <div className="flex h-14 shrink-0 items-center border-b border-neutral-800/60 bg-neutral-900/40">
        {TrackTopBar}
      </div>
      <div className="flex flex-1 min-h-0">
        {TrackContent}
      </div>
      <div className="shrink-0">
        {TrackPlayBar}
      </div>
    </div>
  )
}

function TrackTopBar() {
  const setCurrentState = useStateStore((state) => state.setCurrentState)
  return (
    <div className="flex flex-1 pl-3">
      <button
        onClick={() => { setCurrentState(StateEnum.mainPage) }}
        className="w-10 h-10 flex items-center justify-center rounded-full text-neutral-400 hover:text-white hover:bg-neutral-800/80 transition-colors"
      >
        <ChevronLeft />
      </button>
    </div>
  )
}

export default function TrackPage() {
  return (
    <TrackLayout TrackTopBar={<TrackTopBar />} TrackPlayBar={<MiniPlayer />} TrackContent={<TrackContent />} />
  )
}
