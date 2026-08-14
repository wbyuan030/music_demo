use std::sync::Arc;

use reqwest::Url;
use tauri::State;

use crate::{
    extractor::{bilibili::search::get_video_info, context::ExtractorContext, youtube::api::InnertubeClient},
    global::get_track_state,
    music_fetch::wx::_parse_track_from_wx,
    playback::{BackendRuntime, PlayableEntry, SourceRef},
    types::TrackView,
};

const AUDIOUS_UNSUPPORTED_ERROR: &str =
    "Audius URLs are not supported: track metadata cannot be resolved from a URL";

#[derive(Debug, PartialEq, Eq)]
enum UrlTarget {
    Wechat,
    Youtube { video_id: String },
    Bilibili { bvid: String },
    Audius,
}

/// Parse a supported URL into a validated source target without performing network requests.
fn classify_url(input: &str) -> Result<UrlTarget, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("URL cannot be empty".to_string());
    }

    let parsed = Url::parse(trimmed).map_err(|_| "Invalid URL format".to_string())?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err("Only http(s) URLs are supported".to_string());
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| "URL must include a host".to_string())?
        .to_ascii_lowercase();

    if host == "mp.weixin.qq.com" {
        return Ok(UrlTarget::Wechat);
    }
    if host == "audius.co" || host == "www.audius.co" {
        return Ok(UrlTarget::Audius);
    }
    if host == "youtube.com" || host == "www.youtube.com" || host == "music.youtube.com" {
        let is_watch = parsed.path() == "/watch" || parsed.path() == "/watch/";
        if !is_watch {
            return Err("YouTube URL must use the /watch path".to_string());
        }
        let video_id = parsed
            .query_pairs()
            .find(|(key, _)| key == "v")
            .map(|(_, value)| value.into_owned())
            .ok_or_else(|| "YouTube URL is missing the v video ID".to_string())?;
        return validate_youtube_id(&video_id).map(|()| UrlTarget::Youtube { video_id });
    }
    if host == "youtu.be" {
        let mut segments = parsed
            .path_segments()
            .ok_or_else(|| "YouTube short URL is missing a video ID".to_string())?
            .filter(|segment| !segment.is_empty());
        let video_id = segments
            .next()
            .ok_or_else(|| "YouTube short URL is missing a video ID".to_string())?;
        if segments.next().is_some() {
            return Err("YouTube short URL has an invalid path".to_string());
        }
        validate_youtube_id(video_id)?;
        return Ok(UrlTarget::Youtube {
            video_id: video_id.to_string(),
        });
    }
    if host == "bilibili.com" || host == "www.bilibili.com" {
        let segments: Vec<_> = parsed
            .path_segments()
            .ok_or_else(|| "Bilibili URL is missing a video ID".to_string())?
            .filter(|segment| !segment.is_empty())
            .collect();
        if segments.len() != 2 || segments[0] != "video" {
            return Err("Bilibili URL must use the /video/BV... path".to_string());
        }
        let bvid = normalize_bvid(segments[1])?;
        return Ok(UrlTarget::Bilibili { bvid });
    }

    Err(format!("Unsupported URL host: {host}"))
}

fn validate_youtube_id(video_id: &str) -> Result<(), String> {
    if video_id.len() != 11
        || !video_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
    {
        return Err("Malformed YouTube video ID".to_string());
    }
    Ok(())
}

fn normalize_bvid(candidate: &str) -> Result<String, String> {
    if candidate.len() != 12
        || !candidate.is_ascii()
        || !candidate[2..]
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric())
        || !candidate[..2].eq_ignore_ascii_case("BV")
    {
        return Err("Malformed Bilibili video ID; expected a BV ID".to_string());
    }
    Ok(format!("BV{}", &candidate[2..]))
}

#[tauri::command]
pub async fn parse_track_from_url(
    url: String,
    runtime: State<'_, Arc<BackendRuntime>>,
) -> Result<TrackView, String> {
    let trimmed = url.trim().to_string();
    let target = classify_url(&trimmed)?;
    let entry = match target {
        UrlTarget::Wechat => _parse_track_from_wx(trimmed, &runtime.context)
            .await
            .map_err(|error| error.to_string())?,
        UrlTarget::Youtube { video_id } => parse_youtube_track(&video_id, &runtime.context).await?,
        UrlTarget::Bilibili { bvid } => parse_bilibili_track(&bvid, &runtime.context).await?,
        UrlTarget::Audius => return Err(AUDIOUS_UNSUPPORTED_ERROR.to_string()),
    };

    insert_entry(entry).await
}

async fn parse_youtube_track(
    video_id: &str,
    context: &ExtractorContext,
) -> Result<PlayableEntry, String> {
    let response = InnertubeClient::player(context, video_id)
        .await
        .map_err(|error| format!("YouTube metadata lookup failed: {error}"))?;
    let details = response
        .video_details
        .ok_or_else(|| "YouTube metadata lookup returned no video details".to_string())?;
    let title = details
        .title
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "YouTube metadata has no title".to_string())?;
    let duration = details
        .length_seconds
        .as_deref()
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(0.0);
    let cover_url = details
        .thumbnail
        .and_then(|thumbnail| thumbnail.thumbnails.into_iter().last())
        .map(|thumbnail| thumbnail.url)
        .unwrap_or_else(|| format!("https://i.ytimg.com/vi/{video_id}/hqdefault.jpg"));

    Ok(PlayableEntry {
        view: TrackView::new(
            title,
            details.author.unwrap_or_else(|| "YouTube".to_string()),
            cover_url,
            duration,
            format!("yt:{video_id}"),
        ),
        source_ref: SourceRef::Youtube {
            video_id: video_id.to_string(),
        },
    })
}

async fn parse_bilibili_track(
    bvid: &str,
    context: &ExtractorContext,
) -> Result<PlayableEntry, String> {
    let details = get_video_info(context, bvid)
        .await
        .map_err(|error| format!("Bilibili metadata lookup failed: {error}"))?;
    let title = details
        .title
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "Bilibili metadata has no title".to_string())?;
    let artist = details
        .owner
        .map(|owner| owner.name)
        .unwrap_or_else(|| "Bilibili".to_string());

    Ok(PlayableEntry {
        view: TrackView::new(
            title,
            artist,
            details.pic.unwrap_or_default(),
            details.duration.unwrap_or(0) as f32,
            format!("bili:{bvid}"),
        ),
        source_ref: SourceRef::Bilibili {
            bvid: bvid.to_string(),
        },
    })
}

async fn insert_entry(entry: PlayableEntry) -> Result<TrackView, String> {
    let view = entry.view.clone();
    get_track_state()
        .map_err(|error| error.to_string())?
        .insert(view.id.clone(), entry)
        .await;
    Ok(view)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_supported_url_shapes() {
        assert_eq!(
            classify_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ"),
            Ok(UrlTarget::Youtube {
                video_id: "dQw4w9WgXcQ".to_string()
            })
        );
        assert_eq!(
            classify_url("https://youtu.be/dQw4w9WgXcQ"),
            Ok(UrlTarget::Youtube {
                video_id: "dQw4w9WgXcQ".to_string()
            })
        );
        assert_eq!(
            classify_url("https://www.bilibili.com/video/bv1xx411c7mD"),
            Ok(UrlTarget::Bilibili {
                bvid: "BV1xx411c7mD".to_string()
            })
        );
        assert_eq!(classify_url("https://mp.weixin.qq.com/s/article"), Ok(UrlTarget::Wechat));
    }

    #[test]
    fn rejects_empty_non_http_and_unsupported_urls() {
        assert!(classify_url("  ").unwrap_err().contains("empty"));
        assert!(classify_url("file:///tmp/song").unwrap_err().contains("http"));
        assert!(classify_url("https://example.com/track").unwrap_err().contains("Unsupported"));
    }

    #[test]
    fn rejects_malformed_source_ids() {
        assert!(classify_url("https://youtube.com/watch?v=short").is_err());
        assert!(classify_url("https://youtu.be/not-an-id").is_err());
        assert!(classify_url("https://bilibili.com/video/BV123").is_err());
        assert!(classify_url("https://bilibili.com/video/AV1xx411c7mD").is_err());
    }

    #[test]
    fn classifies_audius_without_guessing_an_id() {
        assert_eq!(classify_url("https://audius.co/artist/song"), Ok(UrlTarget::Audius));
    }

    #[test]
    fn parse_url_command_name_is_stable() {
        assert_eq!(
            __tauri_command_name_parse_track_from_url!(),
            "parse_track_from_url"
        );
    }
}
