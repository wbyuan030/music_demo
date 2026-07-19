pub mod api;
pub mod commands;
pub mod player;
pub mod search;
pub mod types;

use async_trait::async_trait;

use crate::extractor::context::ExtractorContext;
use crate::extractor::model::{RawMediaInfo, Track};
use crate::extractor::protocol::{
    ExtractError, ExtractInput, Extractor, ExtractorResult,
};
use crate::extractor::youtube;

/// YouTube Music extractor.
///
/// Handles:
/// - YouTube Music search (triggered by `ytmusic:search:<query>` URLs)
/// - YouTube Music watch URLs
/// - YouTube Music browse URLs
///
/// Also acts as a standard YouTube extractor for basic video info.
pub struct YouTubeMusicExtractor;

#[async_trait]
impl Extractor for YouTubeMusicExtractor {
    fn key(&self) -> &'static str {
        "youtube:music"
    }

    fn priority(&self) -> i32 {
        10 // Higher priority than default
    }

    fn matches(&self, input: &ExtractInput) -> bool {
        let url = input.url.as_str();
        url.starts_with("ytmusic:")
            || url.contains("music.youtube.com")
            || url.starts_with("ytsearch:")
    }

    async fn extract(
        &self,
        input: ExtractInput,
        context: &ExtractorContext,
    ) -> Result<ExtractorResult, ExtractError> {
        let url = input.url.as_str();

        if url.starts_with("ytmusic:search:") || url.starts_with("ytsearch:") {
            let query = url
                .trim_start_matches("ytmusic:search:")
                .trim_start_matches("ytsearch:")
                .to_string();

            let tracks = youtube::search::search_music(context, &query, Some("songs")).await?;

            let entries: Vec<RawMediaInfo> = tracks
                .into_iter()
                .map(track_to_raw_media)
                .collect();

            return Ok(ExtractorResult::Playlist(crate::extractor::protocol::PlaylistInfo {
                id: None,
                title: Some(format!("Search: {}", query)),
                entries: entries
                    .into_iter()
                    .map(ExtractorResult::Media)
                    .collect(),
                extra: serde_json::Map::new(),
            }));
        }

        // For actual video URLs, extract the video ID and return a redirect
        // to the video extractor (which handles the player API).
        if let Some(video_id) = extract_video_id(url) {
            let info = RawMediaInfo::video(&video_id, "");
            return Ok(ExtractorResult::Media(info));
        }

        Err(ExtractError::Unsupported(url.to_string()))
    }
}

/// Extract a YouTube video ID from various URL formats.
pub fn extract_video_id(url: &str) -> Option<String> {
    // youtu.be/VIDEO_ID
    if let Some(id) = url
        .strip_prefix("https://youtu.be/")
        .or_else(|| url.strip_prefix("http://youtu.be/"))
    {
        return Some(id.split(&['?', '#'][..]).next()?.to_string());
    }

    // youtube.com/watch?v=VIDEO_ID
    if url.contains("youtube.com/watch") || url.contains("music.youtube.com/watch") {
        let query = url.split('?').nth(1)?;
        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            if parts.next()? == "v" {
                return Some(parts.next()?.to_string());
            }
        }
    }

    // youtube.com/embed/VIDEO_ID
    if url.contains("youtube.com/embed/") {
        let id = url.split("/embed/").nth(1)?;
        return Some(id.split(&['?', '#', '/'][..]).next()?.to_string());
    }

    // youtube.com/shorts/VIDEO_ID
    if url.contains("youtube.com/shorts/") {
        let id = url.split("/shorts/").nth(1)?;
        return Some(id.split(&['?', '#', '/'][..]).next()?.to_string());
    }

    None
}

/// Convert an application Track to RawMediaInfo (yt-dlp compatible).
fn track_to_raw_media(track: Track) -> RawMediaInfo {
    let video_id = track
        .id
        .strip_prefix("yt:")
        .unwrap_or(&track.id)
        .to_string();

    let mut extra = serde_json::Map::new();
    if let Some(album) = &track.album {
        extra.insert("album".to_string(), serde_json::Value::String(album.clone()));
    }
    if let Some(dur) = track.duration_ms {
        extra.insert(
            "duration_ms".to_string(),
            serde_json::Value::Number(serde_json::Number::from(dur)),
        );
    }
    if !track.artists.is_empty() {
        extra.insert(
            "artists".to_string(),
            serde_json::Value::Array(
                track
                    .artists
                    .iter()
                    .map(|a| serde_json::Value::String(a.clone()))
                    .collect(),
            ),
        );
    }
    if !track.artwork.is_empty() {
        extra.insert(
            "thumbnail".to_string(),
            serde_json::Value::String(track.artwork[0].url.clone()),
        );
    }

    let mut info = RawMediaInfo::video(&video_id, &track.title);
    info.extra = extra;

    // Set the full YouTube watch URL
    info.extra.insert(
        "webpage_url".to_string(),
        serde_json::Value::String(format!("https://music.youtube.com/watch?v={}", video_id)),
    );
    info.extra.insert(
        "url".to_string(),
        serde_json::Value::String(format!("https://www.youtube.com/watch?v={}", video_id)),
    );

    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_video_id_full_url() {
        assert_eq!(
            extract_video_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn test_extract_video_id_short() {
        assert_eq!(
            extract_video_id("https://youtu.be/dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn test_extract_video_id_music() {
        assert_eq!(
            extract_video_id("https://music.youtube.com/watch?v=dQw4w9WgXcQ&list=RDAMVMdQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn test_extract_video_id_with_params() {
        assert_eq!(
            extract_video_id("https://youtu.be/dQw4w9WgXcQ?t=30"),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn test_extract_video_id_none() {
        assert_eq!(extract_video_id("https://example.com"), None);
    }
}
