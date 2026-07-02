use serde::{Deserialize, Serialize};
use tauri::http::HeaderMap;

use crate::{
    music_fetch::bilibili::{BiliMeta, BiliMetaParse},
    storage::{get_uuid_from_url, TrackDbItem},
};
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum MetaValue {
    Bili(BiliMeta),
    Wechat(String),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TrackMeta {
    pub source: String,
    pub value: MetaValue,
}

impl TrackMeta {
    #[allow(dead_code)]
    pub fn new(source: String, value: MetaValue) -> Self {
        return Self {
            source: source,
            value: value,
        };
    }
}

impl TrackMeta {
    pub async fn parse(&self) -> Option<TrackSrc> {
        match self.source.as_str() {
            "bilibili" => {
                let meta = match self.value.clone() {
                    MetaValue::Bili(d) => d,
                    _ => return None,
                };

                match BiliMetaParse(meta, None).await {
                    Ok(src) => return Some(src),
                    Err(e) => {
                        eprintln!("{}", e);
                        return None;
                    }
                }
            }
            "wechat" => {
                let url = match self.value.clone() {
                    MetaValue::Wechat(d) => Some(TrackSrc::Wechat(d)),
                    _ => return None,
                };
                url
            }
            _ => {
                eprintln!("unknown source:{}", self.source);
                None
            }
        }
    }
}

#[derive(Debug, Serialize, derive_new::new)]
#[serde(rename_all = "camelCase")]
pub struct TrackView {
    pub title: String,
    pub artist: String,
    pub cover_url: String,
    pub duration: f32,
    pub id: String,
}

#[derive(Debug, Clone, derive_new::new)]
pub enum TrackSrc {
    Wechat(String),
    Bilibili(String, HeaderMap),
}

#[derive(Debug, Clone, derive_new::new)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub cover_url: String,
    pub duration: f32,
    pub src: TrackSrc,
    pub meta: TrackMeta,
}

impl Track {
    #[allow(dead_code)]
    pub fn to_track_db(&self) -> TrackDbItem {
        let id = match self.src.clone() {
            TrackSrc::Bilibili(url, _) => get_uuid_from_url(url.as_str()),
            TrackSrc::Wechat(url) => get_uuid_from_url(url.as_str()),
        };
        TrackDbItem {
            title: self.title.clone(),
            artist: self.artist.clone(),
            cover_url: self.cover_url.clone(),
            duration: self.duration.clone(),
            meta: self.meta.clone(),
            id: id.to_string(),
            src: "".to_string(),
        }
    }
}
