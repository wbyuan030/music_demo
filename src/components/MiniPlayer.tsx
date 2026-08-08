import { useEffect } from "react";
import { usePlayerStore } from "../store/Player";
import { formatTime } from "../types/track";
import { Heart, Play, SkipBack, SkipForward, Pause, Loader } from "lucide-react";
import { useStateStore } from "../store/State";
import { StateEnum } from "../types/state";
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'


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
  const onPlaybackStarted = usePlayerStore((state) => state.onPlaybackStarted);
  const onPlaybackEnded = usePlayerStore((state) => state.onPlaybackEnded);
  const onPlaybackFailed = usePlayerStore((state) => state.onPlaybackFailed);
  const onBackendState = usePlayerStore((state) => state.onBackendState);
  const isLoading = usePlayerStore((state) => state.isLoading);

  const setState = useStateStore((state) => state.setCurrentState)
  useEffect(() => {
    let unlistenFuncs: Array<() => void> = [];

    const setup = async () => {
      const p1 = listen<Duration>("play_progress", (event) => {
        setProgress(event.payload.secs as number);
      });

      // 播放状态完全由后端事件驱动，不再本地猜测。
      const p2 = listen("play_start", () => {
        onPlaybackStarted();
      });

      const p3 = listen("play_end", () => {
        onPlaybackEnded();
      });

      const p4 = listen<string>("play_failed", (event) => {
        onPlaybackFailed(event.payload);
      });

      const results = await Promise.all([p1, p2, p3, p4]);
      unlistenFuncs = results;

      // 事件流可能因监听时机丢失（Tauri listen 异步注册）：用快照对账。
      const snapshot = await invoke<import("../types/player").PlaybackStateView>("get_playback_state")
      if (snapshot) onBackendState(snapshot)
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
      return <Loader className="size-5 animate-spin text-emerald-300" />;
    }

    if (isPlaying) {
      return <Pause size={20} className="text-neutral-950 fill-current" />;
    }

    return <Play size={20} className="text-neutral-950 fill-current ml-0.5" />;
  };

  return (
    <div className="fixed bottom-0 left-0 w-full h-20 bg-neutral-900/90 backdrop-blur-xl border-t border-neutral-800 px-4 flex items-center justify-between z-40">
      <div className="flex items-center gap-3 w-1/4 min-w-[140px] max-w-72">
        <button className="relative z-50 group w-12 h-12 shrink-0 hover:scale-105 active:scale-95 transition-transform" onClick={() => {
          setState(StateEnum.detail)
        }}>
          <img
            src={currentTrack.coverUrl}
            referrerPolicy="no-referrer"
            alt={currentTrack.title}
            className={`relative z-10 w-full h-full rounded-lg shadow-lg object-cover ring-1 ring-white/10 transition-all duration-500 ${isPlaying ? 'scale-100' : 'scale-95 opacity-80'}`}
            onError={(e) => { (e.target as HTMLImageElement).style.visibility = 'hidden' }}
          />
          <span className={`absolute inset-0 rounded-lg ring-2 ring-emerald-400/0 transition-all duration-500 ${isPlaying ? 'ring-emerald-400/30' : ''}`} />
        </button>

        <div className="flex flex-col min-w-0 overflow-hidden">
          <span
            className="font-medium text-neutral-100 text-sm truncate leading-tight mb-0.5 [&>em]:text-emerald-400 [&>em]:not-italic"
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

        <div className="flex items-center gap-5 mb-1">
          <button
            onClick={onPrev}
            className="text-neutral-400 hover:text-white transition-colors active:scale-95"
          >
            <SkipBack size={20} className="fill-neutral-400 hover:fill-white" />
          </button>

          <button
            onClick={onTogglePlay}
            className="w-10 h-10 flex items-center justify-center rounded-full bg-emerald-500 hover:bg-emerald-400 active:scale-90 shadow-lg shadow-emerald-500/25 transition-all duration-200 disabled:opacity-50"
          >
            <PlayIcon />
          </button>

          <button
            onClick={onNext}
            className="text-neutral-400 hover:text-white transition-colors active:scale-95"
          >
            <SkipForward size={20} className="fill-neutral-400 hover:fill-white" />
          </button>
        </div>

        <div className="w-full flex items-center gap-2 text-[11px] font-mono text-neutral-500">
          <span className="w-9 text-right">{formatTime(currentTime)}</span>

          <div className="relative flex-1 h-4 flex items-center group">
            <input
              type="range"
              value={currentTime}
              max={currentTrack.duration || 0} // 防止 NaN
              onChange={(e) => onSeek(Number(e.target.value))}
              className="w-full h-1.5 appearance-none cursor-pointer rounded-full bg-neutral-800 group-hover:bg-neutral-700 transition-colors"
            />
          </div>

          <span className="w-9 text-left">{formatTime(currentTrack.duration)}</span>
        </div>
      </div>

      <div className="flex items-center justify-end gap-4 w-1/4">
        <button
          onClick={onToggleLike}
          className={`transition-all active:scale-75 ${isLiked ? 'text-emerald-400' : 'text-neutral-500 hover:text-emerald-300'}`}
        >
          <Heart size={20} fill={isLiked ? "currentColor" : "none"} />
        </button>
      </div>
    </div>
  );
}
