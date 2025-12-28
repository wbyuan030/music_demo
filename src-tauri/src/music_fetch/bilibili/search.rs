use crate::{
    music_fetch::bilibili::{
        types::{MediaItem, MediaQuality},
        utils::{create_comm_head, create_search_head, encode_params, get_cookie},
    },
    types::TrackSrc,
};
use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONNECTION, HOST, REFERER, USER_AGENT};
use serde_json::Value;

//TODO:看起来可以替换掉search global里的逻辑
#[allow(dead_code)]
pub struct SearchRequest {
    keyword: String,
    page: u32,
    page_size: Option<u32>,
    search_type: String,
}
#[allow(dead_code)]
impl SearchRequest {
    pub fn new(keyword: String, page: u32, page_size: Option<u32>, search_type: String) -> Self {
        SearchRequest {
            keyword,
            page,
            page_size,
            search_type,
        }
    }
    async fn build_param(&self) -> Result<String, reqwest::Error> {
        let params = vec![
            ("page", self.page.to_string()),
            ("page_size", self.page_size.unwrap_or(20).to_string()),
            ("order", "".to_string()),
            ("keyword", self.keyword.clone()),
            ("duration", "".to_string()),
            ("tids", "0".to_string()),
            ("search_type", self.search_type.clone()),
        ];
        let encoded_str = encode_params(params).await?;
        println!("{}", encoded_str.clone());
        Ok(encoded_str)
    }
}

pub async fn search_global(keyword: &str) -> Result<Value, reqwest::Error> {
    // TODO: global client
    let headers = create_search_head();
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .cookie_store(true)
        .build()?;
    get_cookie(&client).await?;
    let params = vec![("keyword", keyword.to_string())];
    let encoded_params = encode_params(params).await?;
    let res: Value = client
        .get(format!(
            "https://api.bilibili.com/x/web-interface/wbi/search/all/v2?{}",
            encoded_params
        ))
        .send()
        .await?
        .json()
        .await?;

    Ok(res)
}

async fn get_cid(bvid: Option<&str>, aid: Option<&str>) -> Result<String> {
    let client = reqwest::Client::builder()
        .default_headers(create_comm_head())
        .build()?;

    let url = if let Some(bvid) = bvid {
        format!(
            "https://api.bilibili.com/x/web-interface/view?bvid={}",
            bvid
        )
    } else if let Some(aid) = aid {
        format!("https://api.bilibili.com/x/web-interface/view?aid={}", aid)
    } else {
        return Err(anyhow::anyhow!("input bvid, aid is empty"));
    };

    let res: Value = client.get(url).send().await?.json().await?;

    let cid = res
        .get("data")
        .and_then(|d| d.get("cid"))
        .and_then(|c| c.as_i64())
        .map(|c| c.to_string())
        .ok_or(anyhow::anyhow!("Failed to parse cid"))?;

    Ok(cid)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bilibili_raw_search() {
        let res = search_global("周杰伦").await;
        assert!(res.is_ok());
        let val = res.unwrap();
        assert!(val["data"]["result"].as_array().is_some());
        println!("{:?}", val);
    }

    #[tokio::test]
    async fn test_get_bilibili_media() {
        let item = MediaItem {
            bvid: Some("BV1Yg41167jJ".to_string()), // 让子弹飞
            aid: None,
            cid: None,
        };
        let res = get_media_source(item, MediaQuality::Standard).await;
        // println!("Media Result: {:?}", res);
        assert!(res.is_ok());
    }
}
