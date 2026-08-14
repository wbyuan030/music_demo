import { useState } from "react";
import { usePlayerStore } from "../store/Player";
import { useQueueStore } from "../store/Queue";
import { formatTime } from "../types/track";
import {
  Heart,
  ListMusic,
  Loader,
  Pause,
  Play,
  Repeat,
  Shuffle,
  SkipBack,
  SkipForward,
  Volume2,
  VolumeX,
} from "lucide-react";
import { useStateStore } from "../store/State";
import { StateEnum } from "../types/state";
import { HighlightedText } from "./HighlightedText";
import QueuePanel from "./QueuePanel";

export default function MiniPlayer() {
  const currentTrack = usePlayerStore((state) => state.currentTrack);
  const isPlaying = usePlayerStore((state) => state.isPlaying);
  const isEnded = usePlayerStore((state) => state.isEnded);
  const currentTime = usePlayerStore((state) => state.currentTime);
  const isLiked = usePlayerStore((state) => state.isLiked);
  const isLoading = usePlayerStore((state) => state.isLoading);
  const volume = usePlayerStore((state) => state.volume);
  const isMuted = usePlayerStore((state) => state.isMuted);
  const onTogglePlay = usePlayerStore((state) => state.onTogglePlay);
  const onToggleLike = usePlayerStore((state) => state.onToggleLike);
  const onSeek = usePlayerStore((state) => state.onSeek);
  const onVolumeChange = usePlayerStore((state) => state.onVolumeChange);
  const onToggleMute = usePlayerStore((state) => state.onToggleMute);
  const setState = useStateStore((state) => state.setCurrentState);
  const playNext = useQueueStore((state) => state.playNext);
  const playPrevious = useQueueStore((state) => state.playPrevious);
  const repeatMode = useQueueStore((state) => state.repeatMode);
  const shuffle = useQueueStore((state) => state.shuffle);
  const cycleRepeatMode = useQueueStore((state) => state.cycleRepeatMode);
  const toggleShuffle = useQueueStore((state) => state.toggleShuffle);
  const [isQueueOpen, setQueueOpen] = useState(false);

  if (!currentTrack) {
    return <div className="hidden" />;
  }

  const playIcon = isLoading
    ? <Loader className="size-5 animate-spin text-emerald-300" />
    : isPlaying
      ? <Pause size={20} className="text-neutral-950 fill-current" />
      : <Play size={20} className="text-neutral-950 fill-current ml-0.5" />;

  const repeatLabel = repeatMode === "one"
    ? "单曲循环"
    : repeatMode === "all"
      ? "列表循环"
      : "不循环";

  return (
    <div className="fixed inset-x-0 bottom-0 z-40 grid w-full grid-cols-[minmax(0,1fr)_auto] grid-rows-[auto_auto] gap-x-2 gap-y-1 border-t border-neutral-800 bg-neutral-900/90 px-3 pt-2 safe-area-bottom backdrop-blur-xl md:flex md:h-20 md:items-center md:justify-between md:gap-0 md:px-4 md:py-0">
      <div className="flex min-w-0 items-center gap-2 md:w-1/4 md:max-w-72">
        <button
          className="relative z-50 flex size-12 shrink-0 items-center justify-center touch-manipulation transition-transform hover:scale-105 active:scale-95"
          onClick={() => setState(StateEnum.detail)}
          aria-label="打开当前歌曲详情"
        >
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
            title={currentTrack.title}
          >
            <HighlightedText text={currentTrack.title} />
          </span>
          <span
            className="text-xs text-neutral-400 truncate hover:text-neutral-300 transition-colors cursor-default"
            title={currentTrack.artist}
          >
            {currentTrack.artist}
          </span>
        </div>
      </div>

      <div className="col-span-2 row-start-2 flex min-w-0 w-full flex-col items-center md:col-auto md:flex-1 md:max-w-md md:px-4">
        <div className="mb-1 flex items-center gap-3 sm:gap-5">
          <button
            onClick={playPrevious}
            className="flex size-11 items-center justify-center text-neutral-400 touch-manipulation transition-colors hover:text-white active:scale-95 md:size-auto"
            aria-label="上一首"
          >
            <SkipBack size={20} className="fill-neutral-400 hover:fill-white" />
          </button>

          <button
            onClick={onTogglePlay}
            disabled={isLoading}
            className="flex size-11 items-center justify-center rounded-full bg-emerald-500 shadow-lg shadow-emerald-500/25 transition-all duration-200 hover:bg-emerald-400 active:scale-90 disabled:opacity-50 md:size-10"
            aria-label={isEnded ? "重新播放" : isPlaying ? "暂停" : "播放"}
          >
            {playIcon}
          </button>

          <button
            onClick={playNext}
            className="flex size-11 items-center justify-center text-neutral-400 touch-manipulation transition-colors hover:text-white active:scale-95 md:size-auto"
            aria-label="下一首"
          >
            <SkipForward size={20} className="fill-neutral-400 hover:fill-white" />
          </button>
        </div>

        <div className="flex w-full min-w-0 items-center gap-1 text-[10px] font-mono text-neutral-500 sm:gap-2 sm:text-[11px]">
          <span className="hidden w-7 shrink-0 text-right sm:inline sm:w-9">{formatTime(currentTime)}</span>

          <div className="relative flex h-4 min-w-0 flex-1 items-center group">
            <input
              type="range"
              min={0}
              value={currentTime}
              max={currentTrack.duration || 0}
              disabled={currentTrack.duration <= 0}
              onChange={(e) => void onSeek(Number(e.target.value))}
              aria-label="播放进度"
              className="h-2 w-full appearance-none cursor-pointer rounded-full bg-neutral-800 transition-colors group-hover:bg-neutral-700 disabled:cursor-not-allowed"
            />
          </div>

          <span className="hidden w-7 shrink-0 text-left sm:inline sm:w-9">{formatTime(currentTrack.duration)}</span>
        </div>
      </div>

      <div className="col-start-2 row-start-1 flex min-w-0 max-w-[calc(100vw-9rem)] items-center justify-start gap-0.5 overflow-x-auto overscroll-contain md:col-auto md:w-1/4 md:max-w-none md:min-w-[220px] md:justify-end md:gap-2 md:overflow-visible">
        <button
          onClick={toggleShuffle}
          className={`flex size-11 shrink-0 items-center justify-center rounded-md touch-manipulation transition-colors md:size-auto md:rounded-none ${shuffle ? 'text-emerald-400' : 'text-neutral-500 hover:text-emerald-300'}`}
          aria-label={shuffle ? "关闭随机播放" : "开启随机播放"}
        >
          <Shuffle size={18} />
        </button>
        <button
          onClick={cycleRepeatMode}
          className={`flex size-11 shrink-0 items-center justify-center rounded-md touch-manipulation transition-colors md:size-auto md:rounded-none ${repeatMode === 'off' ? 'text-neutral-500 hover:text-emerald-300' : 'text-emerald-400'}`}
          aria-label={`循环模式：${repeatLabel}`}
          title={repeatLabel}
        >
          <Repeat size={18} />
        </button>
        <button
          onClick={onToggleMute}
          className="flex size-11 shrink-0 items-center justify-center rounded-md text-neutral-500 touch-manipulation transition-colors hover:text-neutral-200 md:size-auto md:rounded-none"
          aria-label={isMuted || volume === 0 ? "取消静音" : "静音"}
        >
          {isMuted || volume === 0 ? <VolumeX size={18} /> : <Volume2 size={18} />}
        </button>
        <input
          type="range"
          min={0}
          max={50}
          value={isMuted ? 0 : volume}
          onChange={(e) => void onVolumeChange(Number(e.target.value))}
          aria-label="音量"
          className="hidden h-1.5 w-20 appearance-none cursor-pointer rounded-full bg-neutral-800 md:block"
        />
        <button
          onClick={() => void onToggleLike()}
          className={`flex size-11 shrink-0 items-center justify-center rounded-md touch-manipulation transition-all active:scale-75 md:size-auto md:rounded-none ${isLiked ? 'text-emerald-400' : 'text-neutral-500 hover:text-emerald-300'}`}
          aria-label={isLiked ? "取消收藏" : "收藏"}
        >
          <Heart size={20} fill={isLiked ? "currentColor" : "none"} />
        </button>
        <button
          type="button"
          onClick={() => setQueueOpen((open) => !open)}
          className={`order-first flex size-11 shrink-0 items-center justify-center rounded-md touch-manipulation transition-colors md:order-none md:size-auto md:rounded-none ${isQueueOpen ? "text-emerald-400" : "text-neutral-500 hover:text-emerald-300"}`}
          aria-label={isQueueOpen ? "关闭播放队列" : "打开播放队列"}
          aria-expanded={isQueueOpen}
        >
          <ListMusic size={19} />
        </button>
      </div>
      {isQueueOpen && <QueuePanel onClose={() => setQueueOpen(false)} />}
    </div>
  );
}
