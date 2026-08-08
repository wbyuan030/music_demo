use serde::{Deserialize, Serialize};
use std::time::SystemTime;

// ── Application-level models ──────────────────────────────────────────

/// Image/thumbnail representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// Application-level Track model — stable, UI-facing.
/// Does NOT expose yt-dlp raw fields directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub title: String,
    pub artists: Vec<String>,
    pub album: Option<String>,
    pub duration_ms: Option<u64>,
    pub artwork: Vec<Image>,
}

/// A playable audio stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStream {
    pub url: String,
    pub mime_type: String,
    pub bitrate: Option<u64>,
    pub codec: Option<String>,
    pub content_length: Option<u64>,
}

/// Playback manifest — what the player needs to play a track.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackManifest {
    pub streams: Vec<AudioStream>,
    pub headers: std::collections::HashMap<String, String>,
    pub expires_at: Option<SystemTime>,
}
