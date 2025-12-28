use serde::Serialize;
use tauri::http::HeaderMap;

#[derive(Debug, Serialize, derive_new::new)]
#[serde(rename_all = "camelCase")]
pub struct TrackView {
    pub title: String,
    pub artist: String,
    pub cover_url: String,
    pub duration: f32,
    pub id: String,
}

#[derive(Debug, Clone)]
pub enum TrackSrc {
    Url(String),
    UrlWithHead(String, HeaderMap),
}

#[derive(Debug)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub cover_url: String,
    pub duration: f32,
    pub src: TrackSrc,
}

impl Track {
    pub fn new(
        title: String,
        artist: String,
        cover_url: String,
        duration: f32,
        src: TrackSrc,
    ) -> Self {
        Self {
            title,
            artist,
            cover_url,
            duration,
            src,
        }
    }
}
