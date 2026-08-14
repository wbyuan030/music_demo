import {
  ChevronDown,
  ChevronUp,
  ListMusic,
  Trash2,
  X,
} from "lucide-react";
import { useQueueStore } from "../store/Queue";
import { formatTime } from "../types/track";

interface QueuePanelProps {
  onClose: () => void;
}

export default function QueuePanel({ onClose }: QueuePanelProps) {
  const queue = useQueueStore((state) => state.queue);
  const currentIndex = useQueueStore((state) => state.currentIndex);
  const playAt = useQueueStore((state) => state.playAt);
  const removeAt = useQueueStore((state) => state.removeAt);
  const move = useQueueStore((state) => state.move);
  const clearQueue = useQueueStore((state) => state.clearQueue);

  return (
    <aside
      className="fixed inset-x-0 bottom-0 z-50 flex max-h-[80vh] w-full flex-col overflow-hidden rounded-t-2xl border border-neutral-700 bg-neutral-900/95 pb-[env(safe-area-inset-bottom)] shadow-2xl shadow-black/50 backdrop-blur-xl md:bottom-[5.25rem] md:right-3 md:left-auto md:max-h-[calc(100vh-6.5rem)] md:w-[min(24rem,calc(100vw-1.5rem))] md:rounded-xl"
      aria-label="播放队列"
    >
      <header className="flex shrink-0 items-center justify-between border-b border-neutral-800 px-4 py-3">
        <div className="flex min-w-0 items-center gap-2">
          <ListMusic size={18} className="text-emerald-400" />
          <h2 className="font-medium text-neutral-100">播放队列</h2>
          <span className="text-xs text-neutral-500">{queue.length} 首</span>
        </div>
        <button
          type="button"
          onClick={onClose}
          className="flex min-h-11 min-w-11 items-center justify-center rounded-md text-neutral-400 transition-colors hover:bg-neutral-800 hover:text-white"
          aria-label="关闭播放队列"
        >
          <X size={18} />
        </button>
      </header>

      <div className="min-h-0 flex-1 overflow-y-auto p-2">
        {queue.length === 0 ? (
          <p className="px-3 py-8 text-center text-sm text-neutral-500">队列为空</p>
        ) : (
          <ol className="space-y-1">
            {queue.map((track, index) => {
              const isCurrent = index === currentIndex;
              return (
                <li
                  key={`${track.id}-${index}`}
                  className={`flex items-center gap-1 rounded-lg p-1.5 transition-colors sm:gap-2 sm:p-2 ${isCurrent ? "bg-emerald-500/10 ring-1 ring-emerald-400/30" : "hover:bg-neutral-800/80"}`}
                >
                  <button
                    type="button"
                    onClick={() => playAt(index)}
                    className="flex min-w-0 flex-1 items-center gap-2 text-left sm:gap-3"
                    aria-label={`播放 ${track.title}`}
                    aria-current={isCurrent ? "true" : undefined}
                  >
                    <span className="w-5 shrink-0 text-center text-xs text-neutral-500">{index + 1}</span>
                    <img
                      src={track.coverUrl}
                      referrerPolicy="no-referrer"
                      alt=""
                      className="size-10 shrink-0 rounded-md object-cover"
                      onError={(event) => { event.currentTarget.style.visibility = "hidden"; }}
                    />
                    <span className="min-w-0 flex-1">
                      <span className={`block truncate text-sm ${isCurrent ? "text-emerald-300" : "text-neutral-200"}`}>
                        {track.title}
                      </span>
                      <span className="block truncate text-xs text-neutral-500">{track.artist}</span>
                    </span>
                    {isCurrent ? (
                      <span className="shrink-0 text-[10px] text-emerald-400">正在播放</span>
                    ) : (
                      <span className="shrink-0 text-xs text-neutral-600">{formatTime(track.duration)}</span>
                    )}
                  </button>

                  <div className="flex shrink-0 items-center gap-0.5">
                    <button
                      type="button"
                      onClick={() => move(index, index - 1)}
                      disabled={index === 0}
                      className="flex size-11 items-center justify-center rounded-md p-0 text-neutral-500 transition-colors hover:bg-neutral-700 hover:text-white disabled:cursor-not-allowed disabled:opacity-25 sm:size-auto sm:p-1"
                      aria-label={`将 ${track.title} 上移`}
                    >
                      <ChevronUp size={16} />
                    </button>
                    <button
                      type="button"
                      onClick={() => move(index, index + 1)}
                      disabled={index === queue.length - 1}
                      className="flex size-11 items-center justify-center rounded-md p-0 text-neutral-500 transition-colors hover:bg-neutral-700 hover:text-white disabled:cursor-not-allowed disabled:opacity-25 sm:size-auto sm:p-1"
                      aria-label={`将 ${track.title} 下移`}
                    >
                      <ChevronDown size={16} />
                    </button>
                    <button
                      type="button"
                      onClick={() => removeAt(index)}
                      className="flex size-11 items-center justify-center rounded-md p-0 text-neutral-500 transition-colors hover:bg-red-500/20 hover:text-red-300 sm:size-auto sm:p-1"
                      aria-label={`移除 ${track.title}`}
                    >
                      <Trash2 size={16} />
                    </button>
                  </div>
                </li>
              );
            })}
          </ol>
        )}
      </div>

      <footer className="flex shrink-0 items-center justify-between border-t border-neutral-800 px-3 py-2">
        <span className="text-xs text-neutral-500">
          {currentIndex >= 0 ? `第 ${currentIndex + 1} 首` : "未选择曲目"}
        </span>
        <button
          type="button"
          onClick={clearQueue}
          disabled={queue.length === 0}
          className="flex min-h-11 items-center gap-1.5 rounded-md px-2 py-1 text-xs text-neutral-400 transition-colors hover:bg-red-500/10 hover:text-red-300 disabled:cursor-not-allowed disabled:opacity-40 sm:min-h-0"
        >
          <Trash2 size={14} />
          清空队列
        </button>
      </footer>
    </aside>
  );
}
