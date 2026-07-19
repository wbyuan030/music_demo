use crate::extractor::bilibili::search::get_video_info;
use crate::extractor::bilibili::types::*;
use crate::extractor::bilibili::utils::bili_headers;
use crate::extractor::context::ExtractorContext;
use crate::extractor::model::{AudioStream, PlaybackManifest};
use crate::extractor::protocol::ExtractError;

/// Get a playable audio manifest for a Bilibili video (BV号).
///
/// Fetches the DASH manifest and extracts the best audio stream.
pub async fn get_video_manifest(
    ctx: &ExtractorContext,
    bvid: &str,
) -> Result<PlaybackManifest, ExtractError> {
    // First get CID
    let info = get_video_info(ctx, bvid).await?;
    let cid = info
        .cid
        .ok_or_else(|| ExtractError::ExtractionFailed("no cid in video info".into()))?;

    // Then get play URL
    let play_url = format!(
        "https://api.bilibili.com/x/player/playurl?bvid={}&cid={}&fnver=0&fnval=4048&fourk=1",
        bvid, cid
    );

    let resp: PlayUrlResponse = ctx
        .http
        .get(&play_url)
        .headers(bili_headers())
        .send()
        .await
        .map_err(|e| ExtractError::NetworkError(e.to_string()))?
        .json()
        .await
        .map_err(|e| ExtractError::ParseError(format!("playurl response: {}", e)))?;

    if resp.code != 0 {
        return Err(ExtractError::ExtractionFailed(format!(
            "Bilibili playurl API error: {} (code {})",
            resp.message, resp.code
        )));
    }

    let data = resp
        .data
        .ok_or_else(|| ExtractError::ExtractionFailed("playurl returned no data".into()))?;

    // Extract audio streams from DASH
    let streams = extract_audio_streams(&data);

    if streams.is_empty() {
        return Err(ExtractError::ExtractionFailed(
            "no audio streams found in Bilibili video".into(),
        ));
    }

    let mut headers = std::collections::HashMap::new();
    headers.insert(
        "Referer".to_string(),
        "https://www.bilibili.com/".to_string(),
    );
    headers.insert(
        "User-Agent".to_string(),
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
    );

    Ok(PlaybackManifest {
        streams,
        headers,
        expires_at: None,
    })
}

/// Extract audio-only streams from Bilibili playurl response.
fn extract_audio_streams(data: &PlayUrlData) -> Vec<AudioStream> {
    let mut streams: Vec<AudioStream> = Vec::new();

    // Fallback to DURL (progressive formats) if no DASH audio
    if let Some(dash) = &data.dash {
        if let Some(audio) = &dash.audio {
            for a in audio {
                let url = a
                    .base_url
                    .as_deref()
                    .or(a.base_url_alt.as_deref())
                    .unwrap_or("")
                    .to_string();

                if url.is_empty() {
                    continue;
                }

                let mime = a.mime_type.as_deref().unwrap_or("audio/mp4").to_string();
                let bitrate = a.bandwidth.map(|b| b as u64);
                let content_length = a.size.map(|s| s as u64);

                streams.push(AudioStream {
                    url,
                    mime_type: mime,
                    bitrate,
                    codec: a.codecs.clone(),
                    content_length,
                });

                // Sort by bitrate descending (best first)
                streams.sort_by(|a, b| {
                    b.bitrate.unwrap_or(0).cmp(&a.bitrate.unwrap_or(0))
                });
            }
        }
    }

    // Fallback to DURL (progressive formats) if no DASH audio
    if streams.is_empty() {
        if let Some(durls) = &data.durl {
            for d in durls {
                if !d.url.is_empty() {
                    streams.push(AudioStream {
                        url: d.url.clone(),
                        mime_type: "video/mp4".to_string(),
                        bitrate: None,
                        codec: None,
                        content_length: d.size.map(|s| s as u64),
                    });
                }
            }
        }
    }

    streams
}
