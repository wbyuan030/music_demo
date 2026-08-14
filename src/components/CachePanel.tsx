import { RefreshCw, Trash2 } from "lucide-react";
import { useEffect } from "react";
import { useCacheStore } from "../store/Cache";

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

export default function CachePanel() {
  const info = useCacheStore((state) => state.info);
  const isLoading = useCacheStore((state) => state.isLoading);
  const isClearing = useCacheStore((state) => state.isClearing);
  const error = useCacheStore((state) => state.error);
  const load = useCacheStore((state) => state.load);
  const clear = useCacheStore((state) => state.clear);

  useEffect(() => {
    void load();
  }, [load]);

  const busy = isLoading || isClearing;
  const handleClear = () => {
    if (busy || !window.confirm("确定要清理可移除的音频缓存吗？正在播放的缓存不会被删除。")) {
      return;
    }
    void clear();
  };

  return (
    <section
      aria-busy={busy}
      className="rounded-xl border border-neutral-800 bg-neutral-900/70 p-4 text-neutral-100"
    >
      <div className="flex items-start justify-between gap-4">
        <div>
          <h2 className="text-sm font-medium">音频缓存</h2>
          <p className="mt-1 text-xs text-neutral-500">仅统计已完成的音频文件</p>
        </div>
        <button
          type="button"
          onClick={() => void load()}
          disabled={busy}
          aria-label="刷新缓存信息"
          className="rounded-md p-2 text-neutral-400 transition-colors hover:bg-neutral-800 hover:text-neutral-100 disabled:pointer-events-none disabled:opacity-50"
        >
          <RefreshCw className={`size-4 ${isLoading ? "animate-spin" : ""}`} />
        </button>
      </div>

      <div className="mt-4 grid grid-cols-2 gap-3">
        <div className="rounded-lg bg-neutral-950/60 px-3 py-2">
          <div className="text-xs text-neutral-500">文件数</div>
          <div className="mt-1 text-lg font-medium">{info?.fileCount ?? "—"}</div>
        </div>
        <div className="rounded-lg bg-neutral-950/60 px-3 py-2">
          <div className="text-xs text-neutral-500">占用空间</div>
          <div className="mt-1 text-lg font-medium">{info ? formatBytes(info.bytes) : "—"}</div>
        </div>
      </div>

      {error && <p className="mt-3 text-xs text-red-400">{error}</p>}

      <button
        type="button"
        onClick={handleClear}
        disabled={busy || info === null || info.fileCount === 0}
        className="mt-4 flex w-full items-center justify-center gap-2 rounded-lg bg-neutral-800 px-3 py-2 text-sm text-neutral-200 transition-colors hover:bg-red-500/20 hover:text-red-300 disabled:pointer-events-none disabled:opacity-50"
      >
        <Trash2 className="size-4" />
        {isClearing ? "清理中…" : "清理缓存"}
      </button>
    </section>
  );
}
