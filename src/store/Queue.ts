import { create } from "zustand";
import type { Track } from "../types/track";
import { usePlayerStore } from "./Player";

export type RepeatMode = "off" | "all" | "one";

interface PlaybackQueueState {
  queue: Track[];
  currentIndex: number;
  repeatMode: RepeatMode;
  shuffle: boolean;
  /** 用一个列表建立播放上下文，并立即播放指定曲目。 */
  playFromList: (track: Track, tracks: Track[]) => void;
  playAt: (index: number) => void;
  removeAt: (index: number) => void;
  move: (fromIndex: number, toIndex: number) => void;
  enqueue: (track: Track) => void;
  clearQueue: () => void;
  playNext: () => void;
  playPrevious: () => void;
  handleEnded: () => void;
  cycleRepeatMode: () => void;
  toggleShuffle: () => void;
}

const QUEUE_STORAGE_KEY = "music_demo.playback_queue";

type PersistedQueue = Pick<PlaybackQueueState, "queue" | "repeatMode" | "shuffle">;

function emptyPersistedQueue(): PersistedQueue {
  return {
    queue: [],
    repeatMode: "off",
    shuffle: false,
  };
}

function getStorage(): Storage | undefined {
  try {
    if (typeof window === "undefined") return undefined;
    return window.localStorage;
  } catch {
    return undefined;
  }
}

function isTrack(value: unknown): value is Track {
  if (typeof value !== "object" || value === null) return false;
  const track = value as Record<string, unknown>;
  return typeof track.title === "string"
    && typeof track.artist === "string"
    && typeof track.coverUrl === "string"
    && typeof track.duration === "number"
    && typeof track.id === "string";
}

function isTrackArray(value: unknown): value is Track[] {
  return Array.isArray(value) && value.every(isTrack);
}

function isRepeatMode(value: unknown): value is RepeatMode {
  return value === "off" || value === "all" || value === "one";
}

function loadPersistedQueue(): PersistedQueue {
  const storage = getStorage();
  if (!storage) return emptyPersistedQueue();

  try {
    const serialized = storage.getItem(QUEUE_STORAGE_KEY);
    if (!serialized) return emptyPersistedQueue();

    const value: unknown = JSON.parse(serialized);
    if (
      typeof value !== "object"
      || value === null
      || !isTrackArray((value as Record<string, unknown>).queue)
      || !isRepeatMode((value as Record<string, unknown>).repeatMode)
      || typeof (value as Record<string, unknown>).shuffle !== "boolean"
    ) {
      return emptyPersistedQueue();
    }

    const persisted = value as Record<string, unknown>;
    return {
      queue: persisted.queue as Track[],
      repeatMode: persisted.repeatMode as RepeatMode,
      shuffle: persisted.shuffle as boolean,
    };
  } catch {
    return emptyPersistedQueue();
  }
}

function persistQueue(state: PersistedQueue): void {
  const storage = getStorage();
  if (!storage) return;

  try {
    storage.setItem(
      QUEUE_STORAGE_KEY,
      JSON.stringify({
        queue: state.queue,
        repeatMode: state.repeatMode,
        shuffle: state.shuffle,
      }),
    );
  } catch {
    // Storage can be unavailable or reject writes in private browsing modes.
  }
}

function playTrack(track: Track | undefined) {
  if (track) void usePlayerStore.getState().setCurrentTrack(track);
}

export const useQueueStore = create<PlaybackQueueState>((set, get) => {
  const restored = loadPersistedQueue();
  const persist = () => persistQueue(get());

  return {
    ...restored,
    currentIndex: -1,

    playFromList: (track, tracks) => {
      const queue = tracks.length > 0 ? tracks : [track];
      const currentIndex = queue.findIndex((item) => item.id === track.id);
      set({
        queue,
        currentIndex: currentIndex >= 0 ? currentIndex : 0,
      });
      persist();
      playTrack(track);
    },
    playAt: (index) => {
      const { queue } = get();
      if (!Number.isInteger(index) || index < 0 || index >= queue.length) return;
      set({ currentIndex: index });
      persist();
      playTrack(queue[index]);
    },

    removeAt: (index) => {
      const { queue, currentIndex } = get();
      if (!Number.isInteger(index) || index < 0 || index >= queue.length) return;

      const nextQueue = queue.filter((_, itemIndex) => itemIndex !== index);
      if (nextQueue.length === 0 || currentIndex < 0) {
        set({ queue: nextQueue, currentIndex: nextQueue.length === 0 ? -1 : currentIndex });
        persist();
        return;
      }

      if (index < currentIndex) {
        set({ queue: nextQueue, currentIndex: currentIndex - 1 });
        persist();
        return;
      }

      if (index > currentIndex) {
        set({ queue: nextQueue, currentIndex });
        persist();
        return;
      }

      const nextIndex = Math.min(index, nextQueue.length - 1);
      set({ queue: nextQueue, currentIndex: nextIndex });
      persist();
      playTrack(nextQueue[nextIndex]);
    },

    move: (fromIndex, toIndex) => {
      const { queue, currentIndex } = get();
      if (
        !Number.isInteger(fromIndex)
        || !Number.isInteger(toIndex)
        || fromIndex < 0
        || fromIndex >= queue.length
        || toIndex < 0
        || toIndex >= queue.length
        || fromIndex === toIndex
      ) return;

      const nextQueue = [...queue];
      const [movedTrack] = nextQueue.splice(fromIndex, 1);
      nextQueue.splice(toIndex, 0, movedTrack);

      let nextCurrentIndex = currentIndex;
      if (currentIndex === fromIndex) {
        nextCurrentIndex = toIndex;
      } else if (fromIndex < currentIndex && toIndex >= currentIndex) {
        nextCurrentIndex = currentIndex - 1;
      } else if (fromIndex > currentIndex && toIndex <= currentIndex) {
        nextCurrentIndex = currentIndex + 1;
      }
      set({ queue: nextQueue, currentIndex: nextCurrentIndex });
      persist();
    },

    enqueue: (track) => {
      const { queue, currentIndex } = get();
      if (queue.some((item) => item.id === track.id)) return;
      const nextQueue = [...queue, track];
      set({
        queue: nextQueue,
        currentIndex: currentIndex >= 0 ? currentIndex : 0,
      });
      persist();
      if (currentIndex < 0) playTrack(track);
    },

    clearQueue: () => {
      set({ queue: [], currentIndex: -1 });
      persist();
    },

    playNext: () => {
      const { queue, currentIndex, repeatMode, shuffle } = get();
      if (queue.length === 0) return;

      if (repeatMode === "one") {
        playTrack(queue[currentIndex]);
        return;
      }

      let nextIndex = -1;
      if (shuffle && queue.length > 1) {
        const candidates = queue
          .map((_, index) => index)
          .filter((index) => index !== currentIndex);
        nextIndex = candidates[Math.floor(Math.random() * candidates.length)] ?? -1;
      } else if (currentIndex + 1 < queue.length) {
        nextIndex = currentIndex + 1;
      } else if (repeatMode === "all") {
        nextIndex = 0;
      }

      if (nextIndex < 0) return;
      set({ currentIndex: nextIndex });
      playTrack(queue[nextIndex]);
    },

    playPrevious: () => {
      const player = usePlayerStore.getState();
      const { queue, currentIndex, repeatMode } = get();
      if (queue.length === 0) return;

      if (!player.isEnded && player.currentTime > 3) {
        void player.onSeek(0);
        return;
      }

      const previousIndex = currentIndex > 0
        ? currentIndex - 1
        : repeatMode === "all"
          ? queue.length - 1
          : currentIndex;
      set({ currentIndex: previousIndex });
      playTrack(queue[previousIndex]);
    },

    handleEnded: () => {
      get().playNext();
    },

    cycleRepeatMode: () => {
      const nextMode: Record<RepeatMode, RepeatMode> = {
        off: "all",
        all: "one",
        one: "off",
      };
      set((state) => ({ repeatMode: nextMode[state.repeatMode] }));
      persist();
    },

    toggleShuffle: () => {
      set((state) => ({ shuffle: !state.shuffle }));
      persist();
    },
  };
});
