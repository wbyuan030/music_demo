import { usePlayerStore } from "../store/Player";
import type { Track } from "../types/track";

export function usePlay(_track: Track) {
  usePlayerStore((state) => state.currentTime)
}
