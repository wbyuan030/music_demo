use std::time::{SystemTime, UNIX_EPOCH};

use crate::extractor::context::ExtractorContext;
use crate::extractor::protocol::ExtractError;

use super::types::*;

const MIXIN_KEY_ENC_TAB: [usize; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35,
    27, 43, 5, 49, 33, 9, 42, 19, 29, 28, 14, 37, 12, 52, 56, 7,
    0, 16, 38, 11, 61, 1, 55, 6, 24, 60, 17, 59, 44, 47, 34, 22,
    40, 57, 62, 41, 51, 40, 13, 20, 63, 21, 39, 26, 36, 25, 54, 30,
];

/// Get or fetch WBI signing keys (with simple in-memory cache).
pub struct WbiKeyCache {
    keys: Option<(String, String)>,
    fetched_at: u64,
}

impl WbiKeyCache {
    pub fn new() -> Self {
        Self { keys: None, fetched_at: 0 }
    }

    pub async fn get_or_fetch(
        &mut self,
        ctx: &ExtractorContext,
    ) -> Result<&(String, String), ExtractError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Cache for 30 seconds (same as yt-dlp)
        if self.keys.is_none() || now - self.fetched_at > 30 {
            let (img_key, sub_key) = fetch_wbi_keys(ctx).await?;
            self.keys = Some((img_key, sub_key));
            self.fetched_at = now;
        }

        Ok(self.keys.as_ref().unwrap())
    }
}

/// Fetch WBI keys from Bilibili nav API.
async fn fetch_wbi_keys(ctx: &ExtractorContext) -> Result<(String, String), ExtractError> {
    let resp: WbiRes = ctx
        .http
        .get("https://api.bilibili.com/x/web-interface/nav")
        .send()
        .await
        .map_err(|e| ExtractError::NetworkError(e.to_string()))?
        .json()
        .await
        .map_err(|e| ExtractError::ParseError(format!("WBI keys: {}", e)))?;

    let img_key = take_filename(resp.data.wbi_img.img_url);
    let sub_key = take_filename(resp.data.wbi_img.sub_url);

    Ok((img_key, sub_key))
}

/// Extract filename from a URL path.
fn take_filename(url: String) -> String {
    url.rsplit('/')
        .next()
        .unwrap_or(&url)
        .split('.')
        .next()
        .unwrap_or(&url)
        .to_string()
}

/// Encode params with WBI signature.
pub fn encode_wbi(
    mut params: Vec<(&str, String)>,
    (img_key, sub_key): &(String, String),
) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mixin_key = get_mixin_key(format!("{}{}", img_key, sub_key).as_bytes());

    params.push(("wts", now.to_string()));

    // Sort params by key
    params.sort_by(|a, b| a.0.cmp(b.0));

    let query: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", url_encode(k), url_encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    let sign = md5::compute(format!("{}{}", query, mixin_key));
    let w_rid = format!("{:x}", sign);

    format!("{}&w_rid={}", query, w_rid)
}

fn get_mixin_key(orig: &[u8]) -> String {
    let mixed: Vec<u8> = MIXIN_KEY_ENC_TAB.iter().map(|&i| orig[i]).collect();
    String::from_utf8_lossy(&mixed).to_string()
}

fn url_encode(s: &str) -> String {
    s.replace('%', "%25")
        .replace(' ', "%20")
        .replace('!', "%21")
        .replace('"', "%22")
        .replace('#', "%23")
        .replace('$', "%24")
        .replace('&', "%26")
        .replace('\'', "%27")
        .replace('(', "%28")
        .replace(')', "%29")
        .replace('*', "%2A")
        .replace('+', "%2B")
        .replace(',', "%2C")
        .replace('/', "%2F")
        .replace(':', "%3A")
        .replace(';', "%3B")
        .replace('=', "%3D")
        .replace('?', "%3F")
        .replace('@', "%40")
        .replace('[', "%5B")
        .replace(']', "%5D")
}

/// Ensure we have a buvid3 cookie for Bilibili API access.
pub async fn ensure_cookie(ctx: &ExtractorContext) -> Result<(), ExtractError> {
    // buvid3 is required by Bilibili API.
    // If not present, we set a random one.
    // Note: reqwest's cookie_store handles this if we set it first.
    // For simplicity, we just make a request to bilibili.com first.
    ctx.http
        .get("https://www.bilibili.com/")
        .send()
        .await
        .map_err(|e| ExtractError::NetworkError(e.to_string()))?;

    Ok(())
}

/// Build standard Bilibili request headers.
pub fn bili_headers() -> reqwest::header::HeaderMap {
    use reqwest::header::*;
    let mut headers = HeaderMap::new();
    headers.insert(REFERER, HeaderValue::from_static("https://www.bilibili.com/"));
    headers.insert(USER_AGENT, HeaderValue::from_static(
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
    ));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json, text/plain, */*"));
    headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
    headers.insert(HOST, HeaderValue::from_static("api.bilibili.com"));
    headers
}
