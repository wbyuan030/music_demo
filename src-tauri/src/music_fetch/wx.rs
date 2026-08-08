use std::sync::Arc;

use anyhow::{Error as InnerError, Result as InnerResult};
use tauri::State;

use crate::{
    extractor::context::ExtractorContext,
    global::get_track_state,
    playback::{BackendRuntime, PlayableEntry, SourceRef},
    storage::get_uuid_from_url,
    types::TrackView,
};
use reqwest::header::{ACCEPT_LANGUAGE, REFERER, USER_AGENT};
use scraper::{Html, Selector};
use tauri::http::{HeaderMap, HeaderValue};

#[tauri::command]
pub async fn parse_track_from_wx(
    url: String,
    runtime: State<'_, Arc<BackendRuntime>>,
) -> Result<TrackView, String> {
    let entry = _parse_track_from_wx(url, &runtime.context)
        .await
        .map_err(|error| error.to_string())?;
    let view = entry.view.clone();
    get_track_state()
        .map_err(|error| error.to_string())?
        .insert(view.id.clone(), entry)
        .await;
    Ok(view)
}

pub async fn _parse_track_from_wx(
    url: String,
    context: &ExtractorContext,
) -> InnerResult<PlayableEntry> {
    let mut header_map = HeaderMap::new();
    header_map.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.0.0.0 Safari/537.36"));
    header_map.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("zh-CN,zh;q=0.9"));
    header_map.insert(REFERER, HeaderValue::from_str(url.as_str())?);

    let resp = context
        .http
        .get(&url)
        .headers(header_map)
        .send()
        .await?
        .text()
        .await?;

    parse_track_from_wx_html(&resp, &url)
}

fn parse_track_from_wx_html(html: &str, url: &str) -> InnerResult<PlayableEntry> {
    let document = Html::parse_document(html);
    let selector =
        Selector::parse("mp-common-mpaudio").map_err(|error| InnerError::msg(error.to_string()))?;
    let element = document
        .select(&selector)
        .next()
        .ok_or(anyhow::anyhow!("找不到mp-common-mp-audio"))?;
    let attrs = element.value();

    let name = attrs.attr("name").unwrap_or("Unknown").to_string();
    let author = attrs.attr("author").unwrap_or("Unknown").to_string();
    let file_id = attrs
        .attr("voice_encode_fileid")
        .ok_or(anyhow::anyhow!("找不到音频id"))?;
    let duration_ms = attrs
        .attr("play_length")
        .unwrap_or("0")
        .parse::<u64>()
        .unwrap_or(0);
    let cover_url =
        "https://images.weserv.nl/?url=".to_string() + attrs.attr("cover").unwrap_or("");
    let audio_url = format!("https://res.wx.qq.com/voice/getvoice?mediaid={}", file_id);

    Ok(PlayableEntry {
        view: TrackView::new(
            name,
            author,
            cover_url,
            duration_ms as f32 / 1000.0,
            get_uuid_from_url(url).to_string(),
        ),
        source_ref: SourceRef::Wechat { url: audio_url },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_track_from_wx() {
        let url = "https://mp.weixin.qq.com/s/test".to_string();
        let html = r#"<mp-common-mpaudio name="Song" author="Artist" voice_encode_fileid="media-id" play_length="1234" cover="cover.jpg"></mp-common-mpaudio>"#;
        let result = parse_track_from_wx_html(html, &url).unwrap();
        assert_eq!(result.view.title, "Song");
        assert_eq!(result.view.artist, "Artist");
        assert_eq!(result.view.duration, 1.234);
        assert_eq!(result.view.id, get_uuid_from_url(&url).to_string());
        assert!(matches!(result.source_ref, SourceRef::Wechat { .. }));
    }
}
