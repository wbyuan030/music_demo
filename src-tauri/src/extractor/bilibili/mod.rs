pub mod player;
pub mod search;
pub mod types;
pub mod utils;

use async_trait::async_trait;

use crate::extractor::context::ExtractorContext;
use crate::extractor::model::{RawMediaInfo, Track};
use crate::extractor::protocol::{
    ExtractError, ExtractInput, Extractor, ExtractorResult,
};

/// Bilibili video/audio extractor.
///
/// Handles:
/// - Bilibili video URLs (BV/AV)
/// - Bilibili search (triggered by `bilisearch:<query>` URLs)
/// - Bilibili audio page URLs (au)
pub struct BiliBiliExtractor;

#[async_trait]
impl Extractor for BiliBiliExtractor {
    fn key(&self) -> &'static str {
        "bilibili"
    }

    fn priority(&self) -> i32 {
        5
    }

    fn matches(&self, input: &ExtractInput) -> bool {
        let url = input.url.as_str();
        url.starts_with("bili:")
            || url.starts_with("bilisearch:")
            || url.contains("bilibili.com/video/")
            || url.contains("bilibili.com/audio/")
    }

    async fn extract(
        &self,
        input: ExtractInput,
        context: &ExtractorContext,
    ) -> Result<ExtractorResult, ExtractError> {
        let url = input.url.as_str();

        if url.starts_with("bilisearch:") {
            let query = url.trim_start_matches("bilisearch:").to_string();
            let tracks = search::search_video(context, &query, 1).await?;

            let entries: Vec<ExtractorResult> = tracks
                .into_iter()
                .map(|t| ExtractorResult::Media(track_to_raw_media(t)))
                .collect();

            return Ok(ExtractorResult::Playlist(
                crate::extractor::protocol::PlaylistInfo {
                    id: None,
                    title: Some(format!("Bilibili Search: {}", query)),
                    entries,
                    extra: serde_json::Map::new(),
                },
            ));
        }

        // Extract BV/AV ID from URL or direct ID
        let video_id = extract_bili_video_id(url);
        if let Some(vid) = video_id {
            let info = RawMediaInfo::video(&vid, "");
            return Ok(ExtractorResult::Media(info));
        }

        Err(ExtractError::Unsupported(url.to_string()))
    }
}

/// Extract a Bilibili video identifier from various formats.
pub fn extract_bili_video_id(url: &str) -> Option<String> {
    // Direct bili:BV1xx or bili:av123
    if let Some(id) = url.strip_prefix("bili:") {
        return Some(id.to_string());
    }

    // bilibili.com/video/BV1xx
    if let Some(idx) = url.find("bilibili.com/video/") {
        let after = &url[idx + "bilibili.com/video/".len()..];
        let id = after.split(&['?', '#', '/'][..]).next().unwrap_or("");
        if !id.is_empty() {
            return Some(id.to_string());
        }
    }

    // bilibili.com/audio/au123
    if let Some(idx) = url.find("bilibili.com/audio/au") {
        let after = &url[idx + "bilibili.com/audio/au".len()..];
        let id = after.split(&['?', '#', '/'][..]).next().unwrap_or("");
        if !id.is_empty() {
            return Some(format!("au{}", id));
        }
    }

    None
}

/// Convert an application Track to RawMediaInfo (yt-dlp compatible).
fn track_to_raw_media(track: Track) -> RawMediaInfo {
    let video_id = track
        .id
        .strip_prefix("bili:")
        .unwrap_or(&track.id)
        .to_string();

    let mut extra = serde_json::Map::new();
    if !track.artists.is_empty() {
        extra.insert(
            "artist".to_string(),
            serde_json::Value::String(track.artists.join(", ")),
        );
    }
    if let Some(dur) = track.duration_ms {
        extra.insert(
            "duration_ms".to_string(),
            serde_json::Value::Number(serde_json::Number::from(dur)),
        );
    }
    if !track.artwork.is_empty() {
        extra.insert(
            "thumbnail".to_string(),
            serde_json::Value::String(track.artwork[0].url.clone()),
        );
    }
    extra.insert(
        "webpage_url".to_string(),
        serde_json::Value::String(format!("https://www.bilibili.com/video/{}", video_id)),
    );

    let mut info = RawMediaInfo::video(&video_id, &track.title);
    info.extra = extra;
    info
}

// ── Tauri commands ────────────────────────────────────────────────────

/// Search Bilibili for videos matching a keyword.
#[tauri::command]
pub async fn search_bilibili_video(keyword: String) -> Result<Vec<TrackView>, String> {
    let ctx = ExtractorContext::new().map_err(|e| e.to_string())?;
    let tracks = search::search_video(&ctx, &keyword, 1)
        .await
        .map_err(|e| e.to_string())?;

    Ok(tracks.into_iter().map(track_to_view).collect())
}

/// Get a playable audio manifest for a Bilibili video.
#[tauri::command]
pub async fn get_bilibili_manifest(bvid: String) -> Result<ManifestResult, String> {
    let ctx = ExtractorContext::new().map_err(|e| e.to_string())?;
    let manifest = player::get_video_manifest(&ctx, &bvid)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ManifestResult {
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
    })
}

// ── Shared view types ─────────────────────────────────────────────────

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackView {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub cover_url: String,
    pub duration: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamInfo {
    pub url: String,
    pub mime_type: String,
    pub bitrate: Option<u64>,
    pub content_length: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManifestResult {
    pub streams: Vec<StreamInfo>,
    pub headers: std::collections::HashMap<String, String>,
}

fn track_to_view(track: Track) -> TrackView {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bili_video_id_bvid() {
        assert_eq!(
            extract_bili_video_id("https://www.bilibili.com/video/BV1GJ411x7"),
            Some("BV1GJ411x7".to_string())
        );
    }

    #[test]
    fn test_extract_bili_video_id_direct() {
        assert_eq!(
            extract_bili_video_id("bili:BV1xx"),
            Some("BV1xx".to_string())
        );
    }

    #[test]
    fn test_extract_bili_video_id_audio() {
        assert_eq!(
            extract_bili_video_id("https://www.bilibili.com/audio/au1003142"),
            Some("au1003142".to_string())
        );
    }

    #[test]
    fn test_parse_bili_duration() {
        assert_eq!(super::search::parse_bili_duration("4:30"), Some(270_000));
        assert_eq!(super::search::parse_bili_duration("1:04:30"), Some(3_870_000));
    }
}
