use reqwest::header::HeaderValue;
use reqwest::header::ACCEPT;
use reqwest::header::USER_AGENT;
use reqwest::header::{HeaderMap, HeaderName};
use reqwest::header::{ACCEPT_ENCODING, ACCEPT_LANGUAGE, ORIGIN, REFERER};
use reqwest::Client;
use reqwest::Error;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct BiliCookieData {
    b_3: String,
    b_4: String,
}

#[derive(Deserialize, Debug)]
struct BiliCookieResponse {
    code: u16,
    message: String,
    data: BiliCookieData,
}

use std::time::{SystemTime, UNIX_EPOCH};

const MIXIN_KEY_ENC_TAB: [usize; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49, 33, 9, 42, 19, 29,
    28, 14, 39, 12, 38, 41, 13, 37, 48, 7, 16, 24, 55, 40, 61, 26, 17, 0, 1, 60, 51, 30, 4, 22, 25,
    54, 21, 56, 59, 6, 63, 57, 62, 11, 36, 20, 34, 44, 52,
];

#[derive(Deserialize)]
struct WbiImg {
    img_url: String,
    sub_url: String,
}

#[derive(Deserialize)]
struct Data {
    wbi_img: WbiImg,
}

#[derive(Deserialize)]
struct ResWbi {
    data: Data,
}

fn get_mixin_key(orig: &[u8]) -> String {
    MIXIN_KEY_ENC_TAB
        .iter()
        .take(32)
        .map(|&i| orig[i] as char)
        .collect::<String>()
}

fn get_url_encoded(s: &str) -> String {
    s.chars()
        .filter_map(|c| match c.is_ascii_alphanumeric() || "-_.~".contains(c) {
            true => Some(c.to_string()),
            false => {
                // 过滤 value 中的 "!'()*" 字符
                if "!'()*".contains(c) {
                    return None;
                }
                let encoded = c
                    .encode_utf8(&mut [0; 4])
                    .bytes()
                    .fold("".to_string(), |acc, b| acc + &format!("%{:02X}", b));
                Some(encoded)
            }
        })
        .collect::<String>()
}

// fn encode_wbi(params: Vec<(&str, String)>, (img_key, sub_key): (String, String)) -> String {
//     let cur_time = match SystemTime::now().duration_since(UNIX_EPOCH) {
//         Ok(t) => t.as_secs(),
//         Err(_) => panic!("SystemTime before UNIX EPOCH!"),
//     };
//     _encode_wbi(params, (img_key, sub_key), cur_time)
// }

pub async fn encode_params(params: Vec<(&str, String)>) -> Result<String, Error> {
    let cur_time = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(t) => t.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };
    let (img_key, sub_key) = get_wbi_keys().await?;
    Ok(_encode_wbi(params, (img_key, sub_key), cur_time))
}

fn _encode_wbi(
    mut params: Vec<(&str, String)>,
    (img_key, sub_key): (String, String),
    timestamp: u64,
) -> String {
    let mixin_key = get_mixin_key((img_key + &sub_key).as_bytes());
    params.push(("wts", timestamp.to_string()));
    params.sort_by(|a, b| a.0.cmp(b.0));
    let query = params
        .iter()
        .map(|(k, v)| format!("{}={}", get_url_encoded(k), get_url_encoded(v)))
        .collect::<Vec<_>>()
        .join("&");
    let web_sign = format!("{:?}", md5::compute(query.clone() + &mixin_key));
    query + &format!("&w_rid={}", web_sign)
}

// TODO: Cache
pub async fn get_wbi_keys() -> Result<(String, String), reqwest::Error> {
    let client = reqwest::Client::new();
    let ResWbi { data:Data{wbi_img} } = client
    .get("https://api.bilibili.com/x/web-interface/nav")
    .header(USER_AGENT,"Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36")
    .header("Referer","https://www.bilibili.com/")
     // SESSDATA=xxxxx
    .header("Cookie", "SESSDATA=xxxxx")
    .send()
    .await?
    .json::<ResWbi>()
    .await?;
    Ok((
        take_filename(wbi_img.img_url).unwrap(),
        take_filename(wbi_img.sub_url).unwrap(),
    ))
}

fn take_filename(url: String) -> Option<String> {
    url.rsplit_once('/')
        .and_then(|(_, s)| s.rsplit_once('.'))
        .map(|(s, _)| s.to_string())
}

pub async fn get_cookie(client: &Client) -> Result<(), reqwest::Error> {
    let _ = client
        .get("https://www.bilibili.com")
        // .header(USER_AGENT,HeaderValue::from_static("Mozilla/5.0 (iPhone; CPU iPhone OS 13_2_3 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/13.0.3 Mobile/15E148 Safari/604.1 Edg/114.0.0.0"))
        .send()
        .await?;
    Ok(())
}

// TODO: Cache
pub fn create_search_head() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/json, text/plain, */*"),
    );
    headers.insert(USER_AGENT,HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36 Edg/89.0.774.63"));
    headers.insert(
        ACCEPT_ENCODING,
        HeaderValue::from_static("gzip, deflate, br"),
    );
    headers.insert(
        ORIGIN,
        HeaderValue::from_static("https://search.bilibili.com"),
    );
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://search.bilibili.com/"),
    );
    headers.insert(
        ACCEPT_LANGUAGE,
        HeaderValue::from_static("zh-CN,zh;q=0.9,en;q=0.8,en-GB;q=0.7,en-US;q=0.6"),
    );

    headers.insert(
        HeaderName::from_static("sec-fetch-site"),
        HeaderValue::from_static("same-site"),
    );
    headers.insert(
        HeaderName::from_static("sec-fetch-mode"),
        HeaderValue::from_static("cors"),
    );
    headers.insert(
        HeaderName::from_static("sec-fetch-dest"),
        HeaderValue::from_static("empty"),
    );
    headers
}
pub fn create_comm_head() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT,HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36 Edg/89.0.774.63"));
    headers.insert(
        ACCEPT_ENCODING,
        HeaderValue::from_static("gzip, deflate, br"),
    );
    headers.insert(
        ACCEPT_LANGUAGE,
        HeaderValue::from_static("zh-CN,zh;q=0.9,en;q=0.8,en-GB;q=0.7,en-US;q=0.6"),
    );
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers
}
