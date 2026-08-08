import type { Track } from "./track";

/** 后端 get_playback_state 快照。 */
export interface PlaybackStateView {
  phase: 'idle' | 'loading' | 'playing' | 'paused';
  trackId: string | null;
  positionSecs: number;
  error: string | null;
}

export interface PlayerState {
  currentTrack: Track | null;
  /** 后端是否正在出声（由 play_start / play_end 驱动） */
  isPlaying: boolean;
  /** 播放请求已发出、后端仍在解析/下载/解码 */
  isLoading: boolean;
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
  /** play_start 事件：后端确认开始播放 */
  onPlaybackStarted: () => void;
  /** play_end 事件：自然结束或手动停止 */
  onPlaybackEnded: () => void;
  /** play_failed 事件：加载/解码失败，携带后端错误信息 */
  onPlaybackFailed: (message: string) => void;
  /** get_playback_state 快照对账：纠正事件流丢失造成的状态偏差 */
  onBackendState: (state: PlaybackStateView) => void;
}
