use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{http::HeaderMap, utils::config::DmgConfig};

use crate::music_fetch::bilibili::{BiliMeta, BiliMetaParse};
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
