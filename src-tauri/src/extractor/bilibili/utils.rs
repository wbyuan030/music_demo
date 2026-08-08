use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::extractor::context::ExtractorContext;
use crate::extractor::protocol::ExtractError;

use super::types::*;

const MIXIN_KEY_ENC_TAB: [usize; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49, 33, 9, 42, 19, 29,
    28, 14, 37, 12, 52, 56, 7, 0, 16, 38, 11, 61, 1, 55, 6, 24, 60, 17, 59, 44, 47, 34, 22, 40, 57,
    62, 41, 51, 40, 13, 20, 63, 21, 39, 26, 36, 25, 54, 30,
];

/// Get or fetch WBI signing keys (process-wide in-memory cache, 30s TTL).
///
/// 进程级共享：Bilibili 的 WBI key 与应用实例无关，跨请求共享避免每次
/// 搜索/播放都重新打 nav 接口。缓存访问不持锁跨 await；nav 瞬时失败时
/// TTL 内的旧 key 降级复用，不硬失败。
pub struct WbiKeyCache;

/// 缓存条目：(img_key, sub_key, fetched_at_unix_secs)
type WbiEntry = (String, String, u64);

static WBI_KEYS: LazyLock<tokio::sync::Mutex<Option<WbiEntry>>> =
    LazyLock::new(|| tokio::sync::Mutex::new(None));

const WBI_TTL_SECS: u64 = 30;

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

impl WbiKeyCache {
    /// 取 WBI 签名 key：TTL 内命中缓存直接返回；未命中/过期则
    /// 释放锁后重新 fetch（double-check），fetch 失败且旧 key 未过期时降级复用。
    pub async fn get_or_fetch(ctx: &ExtractorContext) -> Result<(String, String), ExtractError> {
        let now = unix_now();
        {
            let guard = WBI_KEYS.lock().await;
            if let Some((img_key, sub_key, fetched_at)) = guard.as_ref() {
                if now - *fetched_at <= WBI_TTL_SECS {
                    return Ok((img_key.clone(), sub_key.clone()));
                }
            }
        } // 释放锁，避免持锁跨网络 await

        match fetch_wbi_keys(ctx).await {
            Ok((img_key, sub_key)) => {
                let mut guard = WBI_KEYS.lock().await;
                *guard = Some((img_key.clone(), sub_key.clone(), unix_now()));
                Ok((img_key, sub_key))
            }
            Err(e) => {
                // nav 瞬时失败：TTL 内旧 key 降级复用，避免搜索/播放硬失败
                let guard = WBI_KEYS.lock().await;
                if let Some((img_key, sub_key, fetched_at)) = guard.as_ref() {
                    if now - *fetched_at <= WBI_TTL_SECS {
                        return Ok((img_key.clone(), sub_key.clone()));
                    }
                }
                Err(e)
            }
        }
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
///
/// 进程级一次性：buvid3 只需设置一次（reqwest cookie_store 持久化），
/// 避免每次搜索都打 bilibili.com 首页（旧实现每次调用都请求，
/// 增加延迟与瞬时失败点）。
pub async fn ensure_cookie(ctx: &ExtractorContext) -> Result<(), ExtractError> {
    static COOKIE_DONE: LazyLock<tokio::sync::Mutex<bool>> =
        LazyLock::new(|| tokio::sync::Mutex::new(false));

    {
        let done = COOKIE_DONE.lock().await;
        if *done {
            return Ok(());
        }
    } // 释放锁再打网络

    ctx.http
        .get("https://www.bilibili.com/")
        .send()
        .await
        .map_err(|e| ExtractError::NetworkError(e.to_string()))?;

    *COOKIE_DONE.lock().await = true;
    Ok(())
}

/// Build standard Bilibili request headers.
pub fn bili_headers() -> reqwest::header::HeaderMap {
    use reqwest::header::*;
    let mut headers = HeaderMap::new();
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://www.bilibili.com/"),
    );
    headers.insert(USER_AGENT, HeaderValue::from_static(
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
    ));
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/json, text/plain, */*"),
    );
    headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
    headers.insert(HOST, HeaderValue::from_static("api.bilibili.com"));
    headers
}
