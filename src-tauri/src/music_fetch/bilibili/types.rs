use derive_new::new;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, PartialEq, Clone, Deserialize, Serialize, new)]
pub struct MediaItem {
    pub cid: Option<String>,
    pub bvid: Option<String>,
    pub aid: Option<String>,
}
#[allow(dead_code)]
pub enum MediaQuality {
    Low,
    Standard,
    High,
    Super,
}

#[allow(dead_code)]
impl MediaQuality {
    pub fn to_string(&self) -> String {
        match self {
            MediaQuality::Low => "low".to_string(),
            MediaQuality::Standard => "standard".to_string(),
            MediaQuality::High => "high".to_string(),
            MediaQuality::Super => "super".to_string(),
        }
    }
}

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
