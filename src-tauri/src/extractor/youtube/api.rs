use super::types::*;
use crate::extractor::context::ExtractorContext;
use crate::extractor::protocol::ExtractError;

// Known working InnerTube API key (public, embedded in YouTube web client).
const INNERTUBE_API_KEY: &str = "AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8";

/// InnerTube API client for YouTube/YouTube Music.
pub struct InnertubeClient;

impl InnertubeClient {
    /// Perform a YouTube Music search.
    pub async fn search(
        ctx: &ExtractorContext,
        query: &str,
        params: Option<&str>,
    ) -> Result<InnerTubeResponse, ExtractError> {
        let body = SearchRequest {
            context: InnerTubeContext {
                client: ClientContext {
                    client_name: "WEB_REMIX".to_string(),
                    client_version: "1.20260707.12.00".to_string(),
                    user_agent: None,
                    android_sdk_version: None,
                    os_name: None,
                    os_version: None,
                },
            },
            query: query.to_string(),
            params: params.map(|s| s.to_string()),
        };

        let url = format!(
            "https://music.youtube.com/youtubei/v1/search?key={}&prettyPrint=false",
            INNERTUBE_API_KEY
        );

        let resp = ctx
            .http
            .post(&url)
            .json(&body)
            .header("Origin", "https://music.youtube.com")
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| ExtractError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(ExtractError::ExtractionFailed(format!(
                "search API returned {}",
                resp.status()
            )));
        }

        let data: InnerTubeResponse = resp
            .json()
            .await
            .map_err(|e| ExtractError::ParseError(format!("search response: {}", e)))?;

        if let Some(err) = &data.error {
            return Err(ExtractError::ExtractionFailed(format!(
                "API error [{}]: {}",
                err.code, err.message
            )));
        }

        Ok(data)
    }

    /// Fetch streaming data (audio URLs) for a video.
    pub async fn player(
        ctx: &ExtractorContext,
        video_id: &str,
    ) -> Result<InnerTubeResponse, ExtractError> {
        let body = PlayerRequest {
            context: InnerTubeContext {
                client: ClientContext {
                    client_name: "WEB".to_string(),
                    client_version: "2.20260708.00.00".to_string(),
                    user_agent: None,
                    android_sdk_version: None,
                    os_name: None,
                    os_version: None,
                },
            },
            video_id: video_id.to_string(),
            playback_context: None,
        };

        let url = format!(
            "https://www.youtube.com/youtubei/v1/player?key={}&prettyPrint=false",
            INNERTUBE_API_KEY
        );

        let resp = ctx
            .http
            .post(&url)
            .json(&body)
            .header("Origin", "https://www.youtube.com")
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| ExtractError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(ExtractError::ExtractionFailed(format!(
                "player API returned {}",
                resp.status()
            )));
        }

        let data: InnerTubeResponse = resp
            .json()
            .await
            .map_err(|e| ExtractError::ParseError(format!("player response: {}", e)))?;

        // Check playability
        if let Some(ps) = &data.playability_status {
            if ps.status != "OK" {
                let reason = ps.reason.as_deref().unwrap_or("unknown reason");
                let err = match ps.status.as_str() {
                    "LOGIN_REQUIRED" => ExtractError::RequiresAuth,
                    "AGE_CHECK_REQUIRED" => ExtractError::GeoRestricted,
                    s if s.contains("RATE") || s.contains("LIMIT") => ExtractError::RateLimited,
                    _ => ExtractError::ExtractionFailed(format!(
                        "playability status {}: {}",
                        ps.status, reason
                    )),
                };
                return Err(err);
            }
        }

        if let Some(err) = &data.error {
            return Err(ExtractError::ExtractionFailed(format!(
                "API error [{}]: {}",
                err.code, err.message
            )));
        }

        Ok(data)
    }

    /// Extract audio-only formats from streaming data.
    /// Returns formats sorted by quality (highest bitrate first).
    pub fn extract_audio_formats(
        streaming_data: &StreamingData,
    ) -> Vec<StreamFormat> {
        let mut audio: Vec<StreamFormat> = Vec::new();

        // Check adaptive formats first (primary source for audio-only).
        if let Some(adaptive) = &streaming_data.adaptive_formats {
            for fmt in adaptive {
                if let Some(ref mime) = fmt.mime_type {
                    if mime.starts_with("audio/") {
                        audio.push(fmt.clone());
                    }
                }
            }
        }

        // Also check regular formats.
        if let Some(formats) = &streaming_data.formats {
            for fmt in formats {
                if let Some(ref mime) = fmt.mime_type {
                    if mime.starts_with("audio/") {
                        // Avoid duplicates
                        if !audio.iter().any(|a| a.itag == fmt.itag && a.url == fmt.url) {
                            audio.push(fmt.clone());
                        }
                    }
                }
            }
        }

        // Sort by bitrate descending
        audio.sort_by(|a, b| {
            let a_bitrate = a.average_bitrate.or(a.bitrate).unwrap_or(0);
            let b_bitrate = b.average_bitrate.or(b.bitrate).unwrap_or(0);
            b_bitrate.cmp(&a_bitrate)
        });

        audio
    }
}

/// Parse a duration string like "4:30" or "1:04:30" into milliseconds.
pub fn parse_duration_to_ms(duration_str: &str) -> Option<u64> {
    let parts: Vec<&str> = duration_str.split(':').collect();
    match parts.len() {
        1 => parts[0].parse::<u64>().ok().map(|s| s * 1000),
        2 => {
            let mins = parts[0].parse::<u64>().ok()?;
            let secs = parts[1].parse::<u64>().ok()?;
            Some((mins * 60 + secs) * 1000)
        }
        3 => {
            let hrs = parts[0].parse::<u64>().ok()?;
            let mins = parts[1].parse::<u64>().ok()?;
            let secs = parts[2].parse::<u64>().ok()?;
            Some((hrs * 3600 + mins * 60 + secs) * 1000)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration_to_ms("4:30"), Some(270_000));
        assert_eq!(parse_duration_to_ms("1:04:30"), Some(3_870_000));
        assert_eq!(parse_duration_to_ms("0:05"), Some(5_000));
        assert_eq!(parse_duration_to_ms("30"), Some(30_000));
        assert_eq!(parse_duration_to_ms(""), None);
    }
}

/// Scrape the YouTube watch page for ytInitialPlayerResponse JSON.
/// This is a fallback when the API player endpoint fails.
pub async fn scrape_player_response(
    ctx: &ExtractorContext,
    video_id: &str,
) -> Result<InnerTubeResponse, ExtractError> {
    let url = format!("https://www.youtube.com/watch?v={}", video_id);

    let resp = ctx
        .http
        .get(&url)
        .header("User-Agent", &ctx.options.user_agent)
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
        .await
        .map_err(|e| ExtractError::NetworkError(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(ExtractError::ExtractionFailed(format!(
            "watch page returned {}",
            resp.status()
        )));
    }

    let html = resp
        .text()
        .await
        .map_err(|e| ExtractError::NetworkError(e.to_string()))?;

    // Find ytInitialPlayerResponse in the page HTML
    let start_marker = "ytInitialPlayerResponse = ";
    let start = html
        .find(start_marker)
        .ok_or_else(|| ExtractError::ParseError("ytInitialPlayerResponse not found in page".into()))?
        + start_marker.len();

    let end = html[start..]
        .find("};")
        .map(|i| start + i + 1)
        .ok_or_else(|| ExtractError::ParseError("could not find end of player response".into()))?;

    let json_str = &html[start..=end];
    let data: InnerTubeResponse = serde_json::from_str(json_str)
        .map_err(|e| ExtractError::ParseError(format!("player response JSON: {}", e)))?;

    // Check playability
    if let Some(ps) = &data.playability_status {
        if ps.status != "OK" {
            let reason = ps.reason.as_deref().unwrap_or("unknown reason");
            return Err(ExtractError::ExtractionFailed(format!(
                "playability status {}: {}",
                ps.status, reason
            )));
        }
    }

    Ok(data)
}

/// Check if a format has a ciphered URL that needs deciphering.
pub fn format_requires_decipher(fmt: &StreamFormat) -> bool {
    fmt.url.is_none() && (fmt.cipher.is_some() || fmt.signature_cipher.is_some())
}
