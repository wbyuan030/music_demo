use parking_lot::Mutex;

/// 播放阶段：后端为唯一状态源，前端由事件流 + 快照查询投影。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackPhase {
    Idle,
    Loading,
    Playing,
    Paused,
}

impl PlaybackPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Loading => "loading",
            Self::Playing => "playing",
            Self::Paused => "paused",
        }
    }
}

/// 播放状态快照（对前端公开）。
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackStateView {
    pub phase: String,
    pub track_id: Option<String>,
    pub position_secs: f32,
    pub error: Option<String>,
}

/// 后端播放状态。所有变更发生在事件发射点，保证快照与事件流一致。
pub struct PlaybackStatus {
    phase: Mutex<PlaybackPhase>,
    track_id: Mutex<Option<String>>,
    last_error: Mutex<Option<String>>,
}

impl PlaybackStatus {
    pub fn new() -> Self {
        Self {
            phase: Mutex::new(PlaybackPhase::Idle),
            track_id: Mutex::new(None),
            last_error: Mutex::new(None),
        }
    }

    pub fn set_loading(&self, track_id: &str) {
        *self.phase.lock() = PlaybackPhase::Loading;
        *self.track_id.lock() = Some(track_id.to_string());
        *self.last_error.lock() = None;
    }

    pub fn set_playing(&self) {
        *self.phase.lock() = PlaybackPhase::Playing;
        *self.last_error.lock() = None;
    }

    pub fn set_paused(&self) {
        *self.phase.lock() = PlaybackPhase::Paused;
    }

    pub fn set_idle(&self) {
        *self.phase.lock() = PlaybackPhase::Idle;
    }

    pub fn set_failed(&self, error: String) {
        *self.phase.lock() = PlaybackPhase::Idle;
        *self.last_error.lock() = Some(error);
    }

    pub fn snapshot(&self, position_secs: f32) -> PlaybackStateView {
        PlaybackStateView {
            phase: self.phase.lock().as_str().to_string(),
            track_id: self.track_id.lock().clone(),
            position_secs,
            error: self.last_error.lock().clone(),
        }
    }
}

impl Default for PlaybackStatus {
    fn default() -> Self {
        Self::new()
    }
}
