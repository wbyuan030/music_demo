use crate::music_fetch::bilibili::data::{MediaItem, MediaQuality};
use crate::music_fetch::bilibili::utils::create_search_head;
use crate::types::TrackSrc;
use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONNECTION, HOST, REFERER, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Daum {
    #[serde(rename = "type")]
    pub type_field: String, // 映射 JSON 中的 "type" 关键字
    pub id: i64,
    pub author: String,
    pub mid: i64,
    pub typeid: String,
    pub typename: String,
    pub arcurl: String,
    pub aid: i64,
    pub bvid: String,
    pub title: String,
    pub description: String,
    pub pic: String,
    pub play: i64,
    #[serde(rename = "video_review")]
    pub video_review: i64,
    pub favorites: i64,
    pub tag: String,
    pub review: i64,
    pub pubdate: i64,
    pub senddate: i64,
    pub duration: String,
    pub badgepay: bool,
    #[serde(rename = "hit_columns")]
    pub hit_columns: Vec<String>,
    #[serde(rename = "view_type")]
    pub view_type: String,
    #[serde(rename = "is_pay")]
    pub is_pay: i64,
    #[serde(rename = "is_union_video")]
    pub is_union_video: i64,
    #[serde(rename = "rec_tags")]
    pub rec_tags: Value, // 可能为空或是复杂对象，使用 Value
    #[serde(rename = "new_rec_tags")]
    pub new_rec_tags: Vec<NewRecTag>,
    pub like: i64,
    pub upic: String,
    pub corner: String,
    pub cover: String,
    pub desc: String,
    pub url: String,
    #[serde(rename = "rec_reason")]
    pub rec_reason: String,
    pub danmaku: i64,
    #[serde(rename = "biz_data")]
    pub biz_data: Value,
    #[serde(rename = "is_charge_video")]
    pub is_charge_video: i64,
    pub vt: i64,
    #[serde(rename = "enable_vt")]
    pub enable_vt: i64,
    #[serde(rename = "vt_display")]
    pub vt_display: String,
    pub subtitle: String,
    #[serde(rename = "episode_count_text")]
    pub episode_count_text: String,
    #[serde(rename = "release_status")]
    pub release_status: i64,
    #[serde(rename = "is_intervene")]
    pub is_intervene: i64,
    pub area: i64,
    pub style: i64,
    #[serde(rename = "cate_name")]
    pub cate_name: String,
    #[serde(rename = "is_live_room_inline")]
    pub is_live_room_inline: i64,
    #[serde(rename = "live_status")]
    pub live_status: i64,
    #[serde(rename = "live_time")]
    pub live_time: String,
    pub online: i64,
    #[serde(rename = "rank_index")]
    pub rank_index: i64,
    #[serde(rename = "rank_offset")]
    pub rank_offset: i64,
    pub roomid: i64,
    #[serde(rename = "short_id")]
    pub short_id: i64,
    #[serde(rename = "spread_id")]
    pub spread_id: i64,
    pub tags: String,
    pub uface: String,
    pub uid: i64,
    pub uname: String,
    #[serde(rename = "user_cover")]
    pub user_cover: String,
    #[serde(rename = "parent_area_id")]
    pub parent_area_id: i64,
    #[serde(rename = "parent_area_name")]
    pub parent_area_name: String,
    #[serde(rename = "watched_show")]
    pub watched_show: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewRecTag {
    #[serde(rename = "tag_name")]
    pub tag_name: String,
    #[serde(rename = "tag_style")]
    pub tag_style: i64,
}

async fn get_cid(bvid: Option<&str>, aid: Option<&str>) -> Result<String> {
    let headers = create_search_head();
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .cookie_store(true)
        .build()?;

    let url = if let Some(b) = bvid {
        format!("https://api.bilibili.com/x/web-interface/view?bvid={}", b)
    } else if let Some(a) = aid {
        format!("https://api.bilibili.com/x/web-interface/view?aid={}", a)
    } else {
        return Err(anyhow::anyhow!("input bvid or aid is empty"));
    };

    // 【修改点】这里泛型改为 serde_json::Value
    let res: Value = client.get(url).send().await?.json().await?;

    // 【修改点】像操作 JSON 对象一样提取 data.cid
    // B站 API 返回的 cid 是数字 (i64)，我们需要将其提取出来并转为 String
    let cid = res["data"]["cid"].as_i64().ok_or_else(|| {
        anyhow::anyhow!("Failed to parse CID: field `data.cid` not found or not an integer")
    })?;

    Ok(cid.to_string())
}

pub async fn get_media_source(media: MediaItem, quality: MediaQuality) -> Result<TrackSrc> {
    let cid = match media.cid {
        Some(res) => res,
        None => get_cid(media.bvid.as_deref(), media.aid.as_deref()).await?,
    };

    let bvid = media.bvid.unwrap_or_default();

    let get_url = format!(
        "https://api.bilibili.com/x/player/playurl?bvid={}&cid={}&fnval=16",
        bvid, cid
    );

    let headers = create_search_head();
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let res: Value = client.get(&get_url).send().await?.json().await?;

    let data = res
        .get("data")
        .ok_or_else(|| anyhow::anyhow!("No `data` field in response json"))?;

    let url = if let Some(dash) = data.get("dash") {
        let audios = match dash.get("audio") {
            Some(values) => values.as_array().unwrap(),
            None => return Err(anyhow::anyhow!("No `audio` field in response `dash` json")),
        };

        let index = match quality {
            MediaQuality::Low => 0,
            MediaQuality::Standard => 1,
            MediaQuality::High => 2,
            MediaQuality::Super => 3,
        };

        let audio = audios.get(index).or(audios.last());

        let element = audio.ok_or(anyhow::anyhow!("No audio stream found"))?;

        element
            .get("baseUrl")
            .ok_or(anyhow::anyhow!("No `baseUrl` field in `durl`"))?
            .as_str()
            .ok_or(anyhow::anyhow!("baseUrl is not string"))?
            .to_string()
    } else if let Some(durl) = data.get("durl") {
        let element = durl
            .as_array()
            .ok_or(anyhow::anyhow!("durl is not array"))?
            .first()
            .ok_or(anyhow::anyhow!("durl list is empty"))?;

        element
            .get("url")
            .ok_or(anyhow::anyhow!("No `url` field in `durl`"))?
            .as_str()
            .ok_or(anyhow::anyhow!("url is not string"))?
            .to_string()
    } else {
        return Err(anyhow::anyhow!("No dash or durl found"));
    };

    let mut _headers = HeaderMap::new();

    _headers.insert(
        USER_AGENT,
        HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/237.84.2.178 Safari/537.36")
    );
    _headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    _headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));

    let host_url = url
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(&url)
        .split_once('/')
        .map(|(domain, _)| domain)
        .unwrap_or("");

    if !host_url.is_empty() {
        _headers.insert(HOST, HeaderValue::try_from(host_url)?);
    }

    let refer_tail = if !bvid.is_empty() {
        format!("video/{}", bvid)
    } else {
        String::new()
    };
    let refer_url = format!("https://www.bilibili.com/{}", refer_tail);
    _headers.insert(REFERER, HeaderValue::try_from(refer_url)?);

    Ok(TrackSrc::UrlWithHead(url, _headers))
}
