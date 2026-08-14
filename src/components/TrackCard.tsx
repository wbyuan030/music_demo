import { formatTime, type Track } from "../types/track"
import { Play } from "lucide-react";
import { HighlightedText } from "./HighlightedText";
import { AddToPlaylistButton } from "./AddToPlaylistButton"

export const TrackCard = ({ track, onClick }: { track: Track; onClick: (t: Track) => void }) => {
  return (
    <div
      onClick={() => onClick(track)}
      className="group relative flex min-w-0 w-full touch-manipulation items-center gap-2 rounded-xl border border-neutral-800/80 bg-neutral-800/40 p-3 transition-all duration-300 hover:border-neutral-700 hover:bg-neutral-800 hover:shadow-xl hover:shadow-emerald-500/10 sm:gap-4"
    >
      <div className="relative size-12 shrink-0 overflow-hidden rounded-lg shadow-md sm:size-14">
        <img
          src={track.coverUrl}
          referrerPolicy="no-referrer"
          className="w-full h-full object-cover group-hover:scale-110 transition-transform duration-500"
          alt={track.title}
          onError={(e) => { (e.target as HTMLImageElement).style.visibility = 'hidden' }}
        />
        <div className="absolute inset-0 bg-black/50 opacity-0 group-hover:opacity-100 flex items-center justify-center transition-opacity duration-300">
          <span className="w-9 h-9 rounded-full bg-emerald-500 shadow-lg shadow-emerald-500/40 flex items-center justify-center translate-y-1 group-hover:translate-y-0 transition-transform duration-300">
            <Play size={16} className="fill-neutral-950 text-neutral-950 ml-0.5" />
          </span>
        </div>
      </div>

      <div className="flex-1 min-w-0 flex flex-col gap-1">
        <div className="flex justify-between items-start gap-2">
          <h4
            className="font-medium text-gray-100 text-sm truncate group-hover:text-emerald-300 transition-colors
                       [&>em]:text-emerald-400 [&>em]:not-italic [&>em]:font-bold"
          >
            <HighlightedText text={track.title} />
          </h4>
          <span className="hidden shrink-0 rounded-md bg-neutral-900/80 px-1.5 py-0.5 pt-0.5 text-[11px] font-mono text-neutral-500 sm:inline">
            {formatTime(track.duration)}
          </span>
        </div>
        <p className="text-xs text-neutral-400 truncate group-hover:text-neutral-300 transition-colors">
          {track.artist}
        </p>
      </div>
      <AddToPlaylistButton trackId={track.id} />
    </div>
  );
};
