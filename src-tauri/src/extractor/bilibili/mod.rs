pub mod player;
pub mod search;
pub mod types;
pub mod utils;

use std::sync::Arc;

use tauri::State;

use crate::extractor::model::PlaybackManifest;
use crate::playback::BackendRuntime;

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

// ── Tauri commands ────────────────────────────────────────────────────

/// Get a playable audio manifest for a Bilibili video.
#[tauri::command]
pub async fn get_bilibili_manifest(
    bvid: String,
    runtime: State<'_, Arc<BackendRuntime>>,
) -> Result<PlaybackManifest, String> {
    let bvid = extract_bili_video_id(&bvid).unwrap_or(bvid);
    player::get_video_manifest(&runtime.context, &bvid)
        .await
        .map_err(|e| e.to_string())
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
        assert_eq!(search::parse_bili_duration("4:30"), Some(270_000));
        assert_eq!(search::parse_bili_duration("1:04:30"), Some(3_870_000));
    }
}
