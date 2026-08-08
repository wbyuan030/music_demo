use crate::extractor::bilibili::types::*;
use crate::extractor::bilibili::utils::{bili_headers, encode_wbi, ensure_cookie, WbiKeyCache};
use crate::extractor::context::ExtractorContext;
use crate::extractor::model::{Image, Track};
use crate::extractor::protocol::ExtractError;

/// Search Bilibili for videos matching `keyword`.
/// Returns application-level Track models.
pub async fn search_video(
    ctx: &ExtractorContext,
    keyword: &str,
    page: u32,
) -> Result<Vec<Track>, ExtractError> {
    ensure_cookie(ctx).await?;

    let keys = WbiKeyCache::get_or_fetch(ctx).await?;

    let params = vec![
        ("search_type", "video".to_string()),
        ("keyword", keyword.to_string()),
        ("page", page.to_string()),
    ];
    let query = encode_wbi(params, &keys);

    let url = format!(
        "https://api.bilibili.com/x/web-interface/search/type?{}",
        query
    );

    let resp: BiliSearchResponse = ctx
        .http
        .get(&url)
        .headers(bili_headers())
        .send()
        .await
        .map_err(|e| ExtractError::NetworkError(e.to_string()))?
        .json()
        .await
        .map_err(|e| ExtractError::ParseError(format!("search response: {}", e)))?;

    if resp.code != 0 {
        return Err(ExtractError::ExtractionFailed(format!(
            "Bilibili API error: {} (code {})",
            resp.message, resp.code
        )));
    }

    let results = resp.data.and_then(|d| d.result).unwrap_or_default();

    let tracks: Vec<Track> = results
        .into_iter()
        .map(|r| bili_search_result_to_track(r))
        .collect();

    Ok(tracks)
}

fn bili_search_result_to_track(r: BiliSearchResult) -> Track {
    let video_id = r
        .bvid
        .unwrap_or_else(|| r.aid.map(|a| a.to_string()).unwrap_or_default());
    let id = format!("bili:{}", video_id);

    // Strip HTML tags from title (Bilibili highlights match in <em> tags)
    let title = r
        .title
        .unwrap_or_default()
        .replace("<em class=\"keyword\">", "")
        .replace("</em>", "");

    let artists = if r.author.is_some() {
        vec![r.author.unwrap_or_default()]
    } else {
        vec![]
    };

    let cover_url = r.pic.unwrap_or_default();
    let artwork = if cover_url.is_empty() {
        vec![]
    } else {
        vec![Image {
            url: cover_url,
            width: None,
            height: None,
        }]
    };

    let duration_ms = r.duration.as_deref().and_then(parse_bili_duration);

    Track {
        id,
        title,
        artists,
        album: None,
        duration_ms,
        artwork,
    }
}

/// Parse Bilibili duration format "MM:SS" or "HH:MM:SS" into ms.
pub fn parse_bili_duration(duration: &str) -> Option<u64> {
    let parts: Vec<&str> = duration.split(':').collect();
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

/// Fetch video info (for getting CID).
pub async fn get_video_info(
    ctx: &ExtractorContext,
    bvid: &str,
) -> Result<VideoInfoData, ExtractError> {
    let url = format!(
        "https://api.bilibili.com/x/web-interface/view?bvid={}",
        bvid
    );

    let resp: VideoInfoResponse = ctx
        .http
        .get(&url)
        .headers(bili_headers())
        .send()
        .await
        .map_err(|e| ExtractError::NetworkError(e.to_string()))?
        .json()
        .await
        .map_err(|e| ExtractError::ParseError(format!("video info: {}", e)))?;

    if resp.code != 0 {
        return Err(ExtractError::ExtractionFailed(format!(
            "Bilibili video info API error: {} (code {})",
            resp.message, resp.code
        )));
    }

    resp.data
        .ok_or_else(|| ExtractError::ExtractionFailed("video info returned no data".into()))
}
