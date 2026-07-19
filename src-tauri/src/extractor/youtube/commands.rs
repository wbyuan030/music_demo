use crate::extractor::context::ExtractorContext;
use crate::extractor::model::{PlaybackManifest, Track};
use crate::extractor::youtube;

use serde::{Deserialize, Serialize};

/// Lightweight track result for Tauri serialization.
/// Mirrors the frontend `Track` interface.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackView {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub cover_url: String,
    pub duration: f32, // seconds
}

/// Audio stream info for the player.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamInfo {
    pub url: String,
    pub mime_type: String,
    pub bitrate: Option<u64>,
    pub content_length: Option<u64>,
}

/// Playback manifest for the player.
#[derive(Debug, Serialize, Deserialize)]
pub struct ManifestResult {
    pub streams: Vec<StreamInfo>,
    pub headers: std::collections::HashMap<String, String>,
}

/// Search YouTube Music for tracks matching a keyword.
#[tauri::command]
pub async fn search_youtube_music(
    keyword: String,
) -> Result<Vec<TrackView>, String> {
    let ctx = ExtractorContext::new().map_err(|e| e.to_string())?;
    let tracks = youtube::search::search_music(&ctx, &keyword, Some("songs"))
        .await
        .map_err(|e| e.to_string())?;

    let views: Vec<TrackView> = tracks.into_iter().map(track_to_view).collect();
    Ok(views)
}

/// Get a playable audio manifest for a YouTube video ID or URL.
#[tauri::command]
pub async fn get_youtube_manifest(
    video_id: String,
) -> Result<ManifestResult, String> {
    let ctx = ExtractorContext::new().map_err(|e| e.to_string())?;

    // Extract video ID if a full URL was passed.
    let vid = youtube::extract_video_id(&video_id)
        .unwrap_or_else(|| video_id.clone());

    let manifest = youtube::player::get_manifest(&ctx, &vid)
        .await
        .map_err(|e| e.to_string())?;

    Ok(manifest_to_result(manifest))
}

/// Convert application Track to frontend TrackView.
pub fn track_to_view(track: Track) -> TrackView {
    let artist = track.artists.join(", ");
    let cover_url = track
        .artwork
        .first()
        .map(|img| img.url.clone())
        .unwrap_or_default();
    let duration_secs = track
        .duration_ms
        .map(|ms| ms as f32 / 1000.0)
        .unwrap_or(0.0);

    TrackView {
        id: track.id,
        title: track.title,
        artist,
        cover_url,
        duration: duration_secs,
    }
}

/// Convert internal PlaybackManifest to serializable result.
fn manifest_to_result(manifest: PlaybackManifest) -> ManifestResult {
    ManifestResult {
        streams: manifest
            .streams
            .into_iter()
            .map(|s| StreamInfo {
                url: s.url,
                mime_type: s.mime_type,
                bitrate: s.bitrate,
                content_length: s.content_length,
            })
            .collect(),
        headers: manifest.headers,
    }
}
