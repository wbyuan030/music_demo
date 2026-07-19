use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// yt-dlp compatible raw media info.
/// This is what extractors return directly — keeps yt-dlp field semantics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawMediaInfo {
    pub id: Option<String>,
    pub title: Option<String>,
    pub formats: Vec<RawFormat>,
    pub entries: Vec<RawMediaInfo>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl RawMediaInfo {
    /// Create a minimal video/track entry.
    pub fn video(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: Some(id.into()),
            title: Some(title.into()),
            formats: vec![],
            entries: vec![],
            extra: serde_json::Map::new(),
        }
    }
}

/// A single media format (yt-dlp compatible).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawFormat {
    pub url: Option<String>,
    pub manifest_url: Option<String>,
    pub ext: Option<String>,
    pub format_id: Option<String>,
    pub format_note: Option<String>,

    pub width: Option<i64>,
    pub height: Option<i64>,
    pub tbr: Option<f64>,
    pub abr: Option<f64>,
    pub vbr: Option<f64>,

    pub acodec: Option<String>,
    pub vcodec: Option<String>,
    pub asr: Option<i64>,
    pub audio_channels: Option<i64>,

    pub filesize: Option<i64>,
    pub filesize_approx: Option<i64>,

    pub protocol: Option<String>,
    pub container: Option<String>,

    /// HTTP headers required to access this format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_headers: Option<std::collections::HashMap<String, String>>,

    /// Additional yt-dlp fields preserved for compatibility.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

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
