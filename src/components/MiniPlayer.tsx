import { useEffect } from "react";
import { usePlayerStore } from "../store/Player";
import { formatTime } from "../types/track";
import { Heart, Play, SkipBack, SkipForward, Pause, Loader } from "lucide-react";
import { useStateStore } from "../store/State";
import { StateEnum } from "../types/state";
import { listen } from '@tauri-apps/api/event'


interface Duration {
  secs: number,
  nanos: number
}
export default function MiniPlayer() {
  const currentTrack = usePlayerStore((state) => state.currentTrack);
  const isPlaying = usePlayerStore((state) => state.isPlaying);
  const currentTime = usePlayerStore((state) => state.currentTime);
  const isLiked = usePlayerStore((state) => state.isLiked);
  const onTogglePlay = usePlayerStore((state) => state.onTogglePlay);
  const onToggleLike = usePlayerStore((state) => state.onToggleLike);
  const onPrev = usePlayerStore((state) => state.onPrev);
  const onNext = usePlayerStore((state) => state.onNext);
  const onSeek = usePlayerStore((state) => state.onSeek);
  const setProgress = usePlayerStore((state) => state.setProgress);
  const isLoading = usePlayerStore((state) => state.isLoading);

  const setState = useStateStore((state) => state.setCurrentState)
  useEffect(() => {
    let unlistenFuncs: Array<() => void> = [];

    const setup = async () => {
      const p1 = listen<Duration>("play_progress", (event) => {
        setProgress(event.payload.secs as number);
      });

      const p2 = listen("play_end", (event) => {
        const latestIsPlaying = usePlayerStore.getState().isPlaying;

        if (event.payload == null) {
          if (latestIsPlaying) onTogglePlay();
        } else {
          console.error(event.payload);
        }
      });

      const p3 = listen("play_start", (event) => {
        const latestIsPlaying = usePlayerStore.getState().isPlaying;

        if (event.payload == null) {
          if (!latestIsPlaying) onTogglePlay();
        } else {
          console.error(event.payload);
        }
      });

      const results = await Promise.all([p1, p2, p3]);
      unlistenFuncs = results;
    };

    setup();

    return () => {
      unlistenFuncs.forEach(fn => fn());
    };
  }, []);

  if (!currentTrack) {
    return (
      <div className="hidden">
      </div>
    );
  }

  const PlayIcon = () => {
    if (isLoading) {
      return <Loader className="size-5 animate-spin text-green-400" />;
    }

    if (isPlaying) {
      return <Pause size={20} className="text-green-600 fill-current" />;
    }

    return <Play size={20} className="text-green-600 fill-current" />;
  };

  return (
    <div className="fixed bottom-0 left-0 w-full h-20 bg-neutral-900 border-t border-neutral-800 px-4 flex items-center justify-between z-40 transition-all duration-300">
      <div className="flex items-center gap-3 w-1/4 min-w-[120px] max-w-60">
        <button className="relative z-50 group w-12 h-12 shrink-0 hover:scale-110" onClick={() => {
          setState(StateEnum.detail)
        }}>
          <img
            src={currentTrack.coverUrl}
            referrerPolicy="no-referrer"
            alt={currentTrack.title}

            className={`relative z-10 w-full h-full rounded shadow-lg object-cover transition-transform duration-500 ${isPlaying ? 'scale-100' : 'scale-95 opacity-80'}`}
          />
        </button>

        <div className="flex flex-col min-w-0 overflow-hidden">
          <span
            className="font-medium text-neutral-100 text-sm truncate leading-tight mb-0.5 [&>em]:text-green-400 [&>em]:not-italic"
            dangerouslySetInnerHTML={{ __html: currentTrack.title }}
            title={currentTrack.title}
          />
          <span
            className="text-xs text-neutral-400 truncate hover:text-neutral-300 transition-colors cursor-default"
            title={currentTrack.artist}
          >
            {currentTrack.artist}
          </span>
        </div>
      </div>

      <div className="flex flex-col items-center flex-1 max-w-md px-4">

        <div className="flex items-center gap-6 mb-1">
          <button
            onClick={onPrev}
            className="text-neutral-400 hover:text-white transition-colors active:scale-95"
          >
            <SkipBack size={20} className="text-green-600 fill-green-400 bg-neutral-900" />
          </button>

          <button
            onClick={onTogglePlay}
            className="w-8 h-8 flex items-center justify-center rounded-full hover:scale-105 active:scale-95 transition-all duration-200 bg-neutral-900! fill-green-400! !hover:fill-green-600"
          >
            <PlayIcon />
          </button>

          <button
            onClick={onNext}
            className="text-neutral-400 hover:text-white transition-colors active:scale-95 disabled:scale-75 disabled:color-gray"
          >
            <SkipForward size={20} className="text-green-600 fill-green-400 bg-neutral-900" />
          </button>
        </div>

        <div className="w-full flex items-center gap-2 text-xs font-mono text-neutral-500">
          <span className="w-9 text-right">{formatTime(currentTime)}</span>

          <div className="relative flex-1 h-4 flex items-center group">
            <input
              type="range"
              value={currentTime}
              max={currentTrack.duration || 0} // 防止 NaN
              onChange={(e) => onSeek(Number(e.target.value))}
              className="w-full h-1 bg-neutral-700 rounded-lg appearance-none cursor-pointer accent-white hover:h-1.5 transition-all"
            />
          </div>

          <span className="w-9 text-left">{formatTime(currentTrack.duration)}</span>
        </div>
      </div>

      <div className="flex items-center justify-end gap-4 w-1/4">
        <button
          onClick={onToggleLike}
          className={`transition-transform active:scale-75 ${isLiked ? 'text-green-500' : 'text-neutral-400 hover:text-white'
            }`}
        >
          <Heart size={20} fill={isLiked ? "currentColor" : "red"} className="text-green-900  bg-neutral-900" />
        </button>
      </div>

    </div>
  );
}

