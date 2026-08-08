use serde::{Deserialize, Serialize};

/// 序列化墓碑：仅用于读取迁移前的旧 bilibili 记录（原 MediaItem 字段形状）。
/// 保留变体名与字段形状以保证旧库反序列化兼容；不再有播放路径消费它。
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct LegacyBiliRecord {
    pub cid: Option<String>,
    pub bvid: Option<String>,
    pub aid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum MetaValue {
    Bili(LegacyBiliRecord),
    Wechat(String),
    Extractor(String),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TrackMeta {
    pub source: String,
    pub value: MetaValue,
}

impl TrackMeta {
    pub fn from_source_ref(source_ref: &crate::playback::model::SourceRef) -> Self {
        match source_ref {
            crate::playback::model::SourceRef::Youtube { video_id } => Self {
                source: "extractor".to_string(),
                value: MetaValue::Extractor(format!("yt:{}", video_id)),
            },
            crate::playback::model::SourceRef::Bilibili { bvid } => Self {
                source: "extractor".to_string(),
                value: MetaValue::Extractor(format!("bili:{}", bvid)),
            },
            crate::playback::model::SourceRef::Wechat { url } => Self {
                source: "wechat".to_string(),
                value: MetaValue::Wechat(url.clone()),
            },
            // ==== sync-generated:begin track_meta ====
            crate::playback::model::SourceRef::Audius { id } => Self {
                source: "extractor".to_string(),
                value: MetaValue::Extractor(format!("au:{}", id)),
            },
            // ==== sync-generated:end track_meta ====
        }
    }

    pub fn to_source_ref(&self) -> Option<crate::playback::model::SourceRef> {
        match &self.value {
            MetaValue::Extractor(id) => {
                if let Some(video_id) = id.strip_prefix("yt:") {
                    return Some(crate::playback::model::SourceRef::Youtube {
                        video_id: video_id.to_string(),
                    });
                }
                if let Some(bvid) = id.strip_prefix("bili:") {
                    return Some(crate::playback::model::SourceRef::Bilibili {
                        bvid: bvid.to_string(),
                    });
                }
                // ==== sync-generated:begin track_meta_reverse ====
                if let Some(id) = id.strip_prefix("au:") {
                    return Some(crate::playback::model::SourceRef::Audius { id: id.to_string() });
                }
                // ==== sync-generated:end track_meta_reverse ====
                None
            }
            MetaValue::Wechat(url) => {
                Some(crate::playback::model::SourceRef::Wechat { url: url.clone() })
            }
            // 旧 bilibili 记录：墓碑，序列化兼容但不再可播。
            MetaValue::Bili(_) => None,
        }
    }
}

#[derive(Debug, Serialize, Clone, derive_new::new)]
#[serde(rename_all = "camelCase")]
pub struct TrackView {
    pub title: String,
    pub artist: String,
    pub cover_url: String,
    pub duration: f32,
    pub id: String,
}
