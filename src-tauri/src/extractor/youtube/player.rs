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

    // Build streams from formats that have direct URLs
    let streams: Vec<AudioStream> = audio_formats
        .iter()
        .filter(|f| {
            // Skip formats that need cipher deciphering
            f.url.is_some() && !super::api::format_requires_decipher(f)
        })
        .map(|f| {
            let mime_type = f
                .mime_type
                .as_deref()
                .unwrap_or("audio/webm")
                .to_string();

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
            "found {} audio format(s), but all require signature deciphering (not yet implemented)",
            ciphered_count
        )));
    }

    // Build required HTTP headers for playback.
    let mut headers = std::collections::HashMap::new();
    headers.insert(
        "User-Agent".to_string(),
        ctx.options.user_agent.clone(),
    );
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

/// Validate that an audio URL is accessible (HEAD request).
pub async fn validate_url(
    ctx: &ExtractorContext,
    url: &str,
) -> Result<bool, ExtractError> {
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
