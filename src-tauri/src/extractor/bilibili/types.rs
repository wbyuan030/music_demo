use serde::Deserialize;

/// WBI key response from Bilibili nav API.
#[derive(Debug, Deserialize)]
pub struct WbiRes {
    pub data: WbiData,
}

#[derive(Debug, Deserialize)]
pub struct WbiData {
    #[serde(rename = "wbi_img")]
    pub wbi_img: WbiImg,
}

#[derive(Debug, Deserialize)]
pub struct WbiImg {
    #[serde(rename = "img_url")]
    pub img_url: String,
    #[serde(rename = "sub_url")]
    pub sub_url: String,
}

/// Search API response.
#[derive(Debug, Deserialize)]
pub struct BiliSearchResponse {
    pub code: i64,
    pub message: String,
    pub data: Option<BiliSearchData>,
}

#[derive(Debug, Deserialize)]
pub struct BiliSearchData {
    pub result: Option<Vec<BiliSearchResult>>,
    #[serde(rename = "numResults")]
    pub num_results: Option<i64>,
    pub page: Option<i64>,
    pub pagesize: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct BiliSearchResult {
    pub arcurl: Option<String>,
    pub aid: Option<i64>,
    pub bvid: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    /// URL encoded cover image
    pub pic: Option<String>,
    pub duration: Option<String>,
    pub description: Option<String>,
    pub tag: Option<String>,
    #[serde(rename = "play")]
    pub play_count: Option<i64>,
    #[serde(rename = "video_review")]
    pub comment_count: Option<i64>,
    pub favorites: Option<i64>,
    #[serde(rename = "mid")]
    pub uploader_id: Option<i64>,
    #[serde(rename = "new_rec_tags")]
    pub new_rec_tags: Option<Vec<NewRecTag>>,
}

#[derive(Debug, Deserialize)]
pub struct NewRecTag {
    #[serde(rename = "tag_name")]
    pub tag_name: String,
}

/// Video info response (for CID extraction).
#[derive(Debug, Deserialize)]
pub struct VideoInfoResponse {
    pub code: i64,
    pub message: String,
    pub data: Option<VideoInfoData>,
}

#[derive(Debug, Deserialize)]
pub struct VideoInfoData {
    pub bvid: Option<String>,
    pub aid: Option<i64>,
    pub title: Option<String>,
    pub desc: Option<String>,
    pub pic: Option<String>,
    pub duration: Option<i64>,
    pub owner: Option<VideoOwner>,
    pub stat: Option<VideoStat>,
    pub cid: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct VideoOwner {
    pub mid: i64,
    pub name: String,
    pub face: String,
}

#[derive(Debug, Deserialize)]
pub struct VideoStat {
    pub view: Option<i64>,
    pub like: Option<i64>,
    pub coin: Option<i64>,
    pub favorite: Option<i64>,
    pub share: Option<i64>,
}

/// Play URL response.
#[derive(Debug, Deserialize)]
pub struct PlayUrlResponse {
    pub code: i64,
    pub message: String,
    pub data: Option<PlayUrlData>,
}

#[derive(Debug, Deserialize)]
pub struct PlayUrlData {
    pub durl: Option<Vec<Durl>>,
    pub dash: Option<DashData>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Durl {
    pub url: String,
    pub size: Option<i64>,
    pub length: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct DashData {
    pub video: Option<Vec<DashStream>>,
    pub audio: Option<Vec<DashStream>>,
    #[serde(rename = "flac")]
    pub flac: Option<FlacData>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DashStream {
    pub id: Option<i64>,
    #[serde(rename = "baseUrl")]
    pub base_url: Option<String>,
    #[serde(rename = "base_url")]
    pub base_url_alt: Option<String>,
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    #[serde(rename = "codecs")]
    pub codecs: Option<String>,
    pub bandwidth: Option<i64>,
    pub size: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct FlacData {
    pub audio: Option<DashStream>,
}

/// Audio-only page API response.
#[derive(Debug, Deserialize)]
pub struct AudioInfoResponse {
    pub code: i64,
    pub data: AudioInfoData,
}

#[derive(Debug, Deserialize)]
pub struct AudioInfoData {
    pub title: Option<String>,
    pub author: Option<String>,
    pub cover: Option<String>,
    pub duration: Option<i64>,
    pub cdns: Option<Vec<String>>,
    pub size: Option<i64>,
    pub intro: Option<String>,
    pub passtime: Option<i64>,
    pub uname: Option<String>,
    pub lyric: Option<String>,
}
