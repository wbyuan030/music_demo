
import { formatTime, type Track } from "../types/track"
import { Play } from "lucide-react"; // 假设你安装了 lucide-react 图标库，没有的话用文本或者 svg 代替
export const TrackCard = ({ track, onClick }: { track: Track; onClick: (t: Track) => void }) => {
  return (
    <div
      onClick={() => onClick(track)}
      className="group relative flex items-center gap-4 p-3 rounded-xl 
                 bg-white/5 border border-white/5 
                 hover:bg-white/10 hover:border-white/10 hover:shadow-lg hover:shadow-purple-500/10
                 transition-all duration-300 cursor-pointer w-full"
    >
      <div className="relative w-14 h-14 shrink-0 rounded-lg overflow-hidden shadow-md">
        <img
          src={track.coverUrl}
          referrerPolicy="no-referrer"
          className="w-full h-full object-cover group-hover:scale-110 transition-transform duration-500"
          alt={track.title}
        />
        <div className="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 flex items-center justify-center transition-opacity duration-300">
          <Play size={20} className="fill-white text-white" />
        </div>
      </div>

      <div className="flex-1 min-w-0 flex flex-col gap-1">
        <div className="flex justify-between items-start gap-2">
          <h4
            className="font-medium text-gray-100 text-sm truncate group-hover:text-purple-300 transition-colors 
                       [&>em]:text-purple-400 [&>em]:not-italic [&>em]:font-bold"
            dangerouslySetInnerHTML={{ __html: track.title }}
          />
          <span className="text-xs text-gray-500 font-mono shrink-0 pt-0.5">
            {formatTime(track.duration)}
          </span>
        </div>
        <p className="text-xs text-gray-400 truncate group-hover:text-gray-300">
          {track.artist}
        </p>
      </div>
    </div>
  );
};


