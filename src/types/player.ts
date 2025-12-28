import type { Track } from "./track";

export interface PlayerState {
  currentTrack: Track | null;
  isPlaying: boolean;
  currentTime: number;
  isLiked: boolean;
  clearCurrentTrack: () => void;
  onTogglePlay: () => void;
  onToggleLike: () => void;
  onNext: () => void;
  onPrev: () => void;
  onSeek: (time: number) => void;
  setCurrentTrack: (track: Track) => void;
  setProgress: (t: number) => void;
}
