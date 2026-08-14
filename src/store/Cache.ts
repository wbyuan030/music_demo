import { create } from "zustand";
import { safeInvoke } from "../services/invoke";

export interface CacheInfo {
  fileCount: number;
  bytes: number;
}

interface CacheState {
  info: CacheInfo | null;
  isLoading: boolean;
  isClearing: boolean;
  error: string | null;
  load: () => Promise<boolean>;
  clear: () => Promise<boolean>;
}

export const useCacheStore = create<CacheState>((set) => ({
  info: null,
  isLoading: false,
  isClearing: false,
  error: null,
  load: async () => {
    set({ isLoading: true, error: null });
    const info = await safeInvoke<CacheInfo>("get_cache_info");
    if (info === null) {
      set({ isLoading: false, error: "无法读取缓存信息" });
      return false;
    }
    set({ info, isLoading: false, error: null });
    return true;
  },
  clear: async () => {
    set({ isClearing: true, error: null });
    const info = await safeInvoke<CacheInfo>("clear_cache");
    if (info === null) {
      set({ isClearing: false, error: "清理缓存失败" });
      return false;
    }
    set({ info, isClearing: false, error: null });
    return true;
  },
}));
