use crate::music_fetch::bilibili::utils::{
    create_comm_head, create_search_head, encode_params, get_cookie,
};
use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONNECTION, HOST, REFERER, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ==========================================
// 数据结构定义
// ==========================================

pub struct SearchRequest {
    keyword: String,
    page: u32,
    page_size: Option<u32>,
    search_type: String,
}
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

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    pub bvid: Option<String>,
    pub aid: Option<String>,
    pub cid: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum MediaQuality {
    Low,      // 对应 TS low
    Standard, // 对应 TS standard
    High,     // 对应 TS high
    Super,    // 对应 TS super
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

    println!("{:?}", res);
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

pub async fn get_media_source(
    media: MediaItem,
    quality: MediaQuality,
) -> Result<(String, HeaderMap)> {
    let cid = match media.cid {
        Some(res) => res,
        None => get_cid(media.bvid.as_deref(), media.aid.as_deref()).await?,
    };

    let client = reqwest::Client::builder()
        .default_headers(create_comm_head())
        .build()?;

    let mut params = Vec::new();
    if let Some(ref bvid) = media.bvid {
        params.push(("bvid", bvid.as_str()));
    } else if let Some(ref aid) = media.aid {
        params.push(("aid", aid.as_str()));
    } else {
        return Err(anyhow::anyhow!("input bvid, aid is empty"));
    }
    params.push(("cid", cid.as_str()));
    params.push(("fnval", "16"));

    let res: Value = client
        .get("https://api.bilibili.com/x/player/playurl")
        .query(&params)
        .send()
        .await?
        .json()
        .await?;

    let data = res
        .get("data")
        .ok_or(anyhow::anyhow!("No `data` field in response"))?;

    let url = if let Some(dash) = data.get("dash") {
        let mut audios = dash
            .get("audio")
            .and_then(|v| v.as_array())
            .cloned()
            .ok_or(anyhow::anyhow!("No `audio` field in dash"))?;

        audios.sort_by_key(|a| a.get("bandwidth").and_then(|v| v.as_u64()).unwrap_or(0));

        let index = match quality {
            MediaQuality::Low => 0,
            MediaQuality::Standard => 1,
            MediaQuality::High => 2,
            MediaQuality::Super => 3,
        };

        let audio = audios
            .get(index)
            .or(audios.last())
            .ok_or(anyhow::anyhow!("No audio stream found"))?;

        audio
            .get("baseUrl")
            .or(audio.get("base_url"))
            .and_then(|v| v.as_str())
            .ok_or(anyhow::anyhow!("No `baseUrl` found"))?
            .to_string()
    } else if let Some(durls) = data.get("durl") {
        durls
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v.get("url"))
            .and_then(|v| v.as_str())
            .ok_or(anyhow::anyhow!("No `url` field in durl"))?
            .to_string()
    } else {
        return Err(anyhow::anyhow!("No dash or durl found"));
    };

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36 Edg/89.0.774.63"));
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(
        "accept-encoding",
        HeaderValue::from_static("gzip, deflate, br"),
    );
    headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));

    if let Some(host_str) = url.split("://").nth(1).and_then(|s| s.split('/').next()) {
        if let Ok(val) = HeaderValue::from_str(host_str) {
            headers.insert(HOST, val);
        }
    }

    let refer_id = media.bvid.or(media.aid).unwrap_or_default();
    let referer = format!("https://www.bilibili.com/video/{}", refer_id);
    if let Ok(val) = HeaderValue::from_str(&referer) {
        headers.insert(REFERER, val);
    }

    Ok((url, headers))
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
        let (url, headers) = res.unwrap();
        assert!(url.contains("http"));
        assert!(headers.contains_key("referer"));
    }
}
