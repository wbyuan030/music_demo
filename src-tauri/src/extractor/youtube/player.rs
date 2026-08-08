use crate::extractor::context::ExtractorContext;
use crate::extractor::model::{AudioStream, PlaybackManifest};
use crate::extractor::protocol::ExtractError;

use super::api::InnertubeClient;

/// Strategy:
/// 1. Try InnerTube player API first
/// 2. Fall back to scraping ytInitialPlayerResponse from the watch page
/// 3. Filter to audio-only adaptive formats
///
/// Note: Some formats may have ciphered URLs requiring signature deciphering
/// (not yet implemented). Those formats are excluded from the manifest.
pub async fn get_manifest(
    ctx: &ExtractorContext,
    video_id: &str,
) -> Result<PlaybackManifest, ExtractError> {
    // Try API first; if it returns no streaming data, fall back to page scraping
    let resp = match InnertubeClient::player(ctx, video_id).await {
        Ok(r) => {
            if r.streaming_data.is_some() {
                r
            } else {
                match super::api::scrape_player_response(ctx, video_id).await {
                    Ok(scraped) => scraped,
                    Err(_) => r, // Keep API response for better error message
                }
            }
        }
        Err(_) => super::api::scrape_player_response(ctx, video_id).await?,
    };

    let streaming_data = resp
        .streaming_data
        .ok_or_else(|| ExtractError::ExtractionFailed(
            "no streaming data available for this video (may require authentication/signature deciphering)".into()
        ))?;

    let audio_formats = InnertubeClient::extract_audio_formats(&streaming_data);

    if audio_formats.is_empty() {
        return Err(ExtractError::ExtractionFailed(
            "no audio formats available for this video".into(),
        ));
    }

    // Rodio/Symphonia supports the AAC/MP4 and MP3 families used below, but
    // does not ship an Opus decoder. Do not hand WebM/Opus URLs to playback.
    let streams: Vec<AudioStream> = audio_formats
        .iter()
        .filter(|f| {
            f.url.is_some()
                && !super::api::format_requires_decipher(f)
                && is_rodio_playable(f.mime_type.as_deref())
        })
        .map(|f| {
            let mime_type = f.mime_type.as_deref().unwrap_or("audio/webm").to_string();

            let content_length = f
                .content_length
                .as_ref()
                .and_then(|s| s.parse::<u64>().ok());

            let bitrate = f.average_bitrate.or(f.bitrate).map(|b| b as u64);

            AudioStream {
                url: f.url.clone().unwrap_or_default(),
                mime_type,
                bitrate,
                codec: None,
                content_length,
            }
        })
        .collect();

    if streams.is_empty() {
        let ciphered_count = audio_formats
            .iter()
            .filter(|f| super::api::format_requires_decipher(f))
            .count();

        return Err(ExtractError::ExtractionFailed(format!(
            "found {} audio format(s), but none are directly playable by rodio ({} require signature deciphering)",
            audio_formats.len(),
            ciphered_count
        )));
    }

    // Build required HTTP headers for playback.
    let mut headers = std::collections::HashMap::new();
    headers.insert("User-Agent".to_string(), ctx.options.user_agent.clone());
    headers.insert("Origin".to_string(), "https://www.youtube.com".to_string());
    headers.insert(
        "Referer".to_string(),
        format!("https://www.youtube.com/watch?v={}", video_id),
    );

    // Expiry from streaming data.
    let expires_at = streaming_data
        .expires_in_seconds
        .as_ref()
        .and_then(|s| s.parse::<u64>().ok())
        .map(|secs| {
            std::time::SystemTime::now()
                .checked_add(std::time::Duration::from_secs(secs))
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        });

    Ok(PlaybackManifest {
        streams,
        headers,
        expires_at,
    })
}

fn is_rodio_playable(mime_type: Option<&str>) -> bool {
    let Some(mime_type) = mime_type else {
        return false;
    };
    let normalized = mime_type.to_ascii_lowercase();
    if normalized.contains("opus") {
        return false;
    }
    matches!(
        normalized.split(';').next().map(str::trim),
        Some("audio/mp4")
            | Some("audio/mpeg")
            | Some("audio/flac")
            | Some("audio/ogg")
            | Some("audio/wav")
            | Some("audio/x-wav")
    )
}

/// Validate that an audio URL is accessible (HEAD request).
pub async fn validate_url(ctx: &ExtractorContext, url: &str) -> Result<bool, ExtractError> {
    let resp = ctx
        .http
        .head(url)
        .header("User-Agent", &ctx.options.user_agent)
        .header("Range", "bytes=0-0")
        .send()
        .await
        .map_err(|e| ExtractError::NetworkError(e.to_string()))?;

    Ok(resp.status().is_success() || resp.status().as_u16() == 206)
}

#[cfg(test)]
mod tests {
    use super::is_rodio_playable;

    #[test]
    fn accepts_rodio_supported_audio_formats() {
        assert!(is_rodio_playable(Some("audio/mp4; codecs=\"mp4a.40.2\"")));
        assert!(is_rodio_playable(Some("audio/mpeg")));
        assert!(is_rodio_playable(Some("audio/ogg; codecs=\"vorbis\"")));
    }

    #[test]
    fn rejects_webm_opus_and_unknown_audio_formats() {
        assert!(!is_rodio_playable(Some("audio/webm; codecs=\"opus\"")));
        assert!(!is_rodio_playable(Some("audio/ogg; codecs=\"opus\"")));
        assert!(!is_rodio_playable(None));
    }
}
