use crate::music::TrackStore;
use crate::storage::get_uuid_from_url;
use crate::types::{MetaValue, Track, TrackMeta, TrackSrc, TrackView};
use anyhow::{Error as InnerError, Result as InnerResult};
use reqwest::header::{ACCEPT_LANGUAGE, REFERER, USER_AGENT};
use scraper::{Html, Selector};
use tauri::http::{HeaderMap, HeaderValue};

#[tauri::command]
pub async fn parse_track_from_wx(
    state: tauri::State<'_, TrackStore>,
    // db: tauri::State<'_, Arc<SyncMutex<Database<'static>>>>,
    url: String,
) -> Result<TrackView, String> {
    let view_id = get_uuid_from_url(url.as_ref()).to_string();
    let _track = _parse_track_from_wx(url).await;
    let track = match _track {
        Ok(t) => t,
        Err(e) => return Err(e.to_string()),
    };
    let track_view = TrackView::new(
        track.title.clone(),
        track.artist.clone(),
        track.cover_url.clone(),
        track.duration.clone(),
        view_id.clone(),
    );
    state.tracks.lock().unwrap().insert(view_id, track);
    // let db = db.lock().unwrap();
    // // _add_recent_track(db, track);
    Ok(track_view)
}

pub async fn _parse_track_from_wx(url: String) -> InnerResult<Track> {
    let mut header_map = HeaderMap::new();
    header_map.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.0.0.0 Safari/537.36"));
    header_map.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("zh-CN,zh;q=0.9"));
    header_map.insert(REFERER, HeaderValue::from_str(url.as_str())?);
    let client = reqwest::Client::builder()
        .default_headers(header_map)
        .build()?;

    let resp = client.get(url).send().await?.text().await?;

    let document = Html::parse_document(resp.as_str());
    let selector_res = Selector::parse("mp-common-mpaudio");
    let selector = match selector_res {
        Ok(v) => v,
        Err(e) => return Err(InnerError::msg(e.to_string())),
    };
    let element = document
        .select(&selector)
        .next()
        .ok_or(anyhow::anyhow!("找不到mp-common-mp-audio"))?;

    let attrs = element.clone().value();

    let name = attrs.attr("name").unwrap_or("Unknown").to_string();
    let author = attrs.attr("author").unwrap_or("Unknown").to_string();
    let file_id = attrs
        .attr("voice_encode_fileid")
        .ok_or(anyhow::anyhow!("找不到音频id"))?;

    let duration_str = attrs.attr("play_length").unwrap_or("0");
    let duration_ms = duration_str.parse::<u64>().unwrap_or(0);
    let duration = duration_ms as f32 / 1000.0;
    let cover_url =
        "https://images.weserv.nl/?url=".to_string() + attrs.attr("cover").unwrap_or("").as_ref();
    let meta = TrackMeta {
        source: "wechat".to_string(),
        value: MetaValue::Wechat(
            "https://res.wx.qq.com/voice/getvoice?mediaid=".to_string() + file_id,
        ),
    };
    Ok(Track::new(
        name.clone(),
        author.clone(),
        cover_url.clone(),
        duration,
        TrackSrc::Wechat("https://res.wx.qq.com/voice/getvoice?mediaid=".to_string() + file_id),
        meta,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_parse_track_from_wx() {
        let url = "https://mp.weixin.qq.com/s/f0wT3dQ1dK-9EpM2kEsYwg".to_string();
        let result = _parse_track_from_wx(url.to_string()).await;
        assert!(result.is_ok(), "测试失败，报错信息:{}", result.unwrap_err());
    }
}
