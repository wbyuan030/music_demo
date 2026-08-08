use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;

use crate::extractor::{context::ExtractorContext, model::Track};
use crate::types::TrackView;

use super::{
    model::{PlayableEntry, SourceKind, SourceRef},
    resolver::PlaybackError,
};

/// 搜索能力抽象：来源 adapter 实现它，供 `search_music` command 合并。
/// 每个来源同时实现 `PlaybackResolver` 完成播放，两个 trait 共用一个 adapter 类型。
#[async_trait]
pub trait SearchProvider: Send + Sync {
    fn source(&self) -> SourceKind;

    async fn search(
        &self,
        keyword: &str,
        context: &ExtractorContext,
    ) -> Result<Vec<PlayableEntry>, PlaybackError>;
}

/// 按来源注册搜索能力，与 `ResolverRegistry` 对称。
/// 同一 adapter 类型会同时注册到两个 registry。
/// `order` 保持注册顺序，供 `search_music` 的“全部”分发使用。
pub struct SearchRegistry {
    providers: HashMap<SourceKind, Arc<dyn SearchProvider>>,
    order: Vec<SourceKind>,
}

impl SearchRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            order: Vec::new(),
        }
    }

    pub fn register<R>(&mut self, provider: R)
    where
        R: SearchProvider + 'static,
    {
        let kind = provider.source();
        if !self.order.contains(&kind) {
            self.order.push(kind);
        }
        self.providers.insert(kind, Arc::new(provider));
    }

    /// 已注册的来源，按注册顺序。
    pub fn sources(&self) -> &[SourceKind] {
        &self.order
    }

    pub async fn search(
        &self,
        source: SourceKind,
        keyword: &str,
        context: &ExtractorContext,
    ) -> Result<Vec<PlayableEntry>, PlaybackError> {
        let provider = self.providers.get(&source).ok_or_else(|| {
            PlaybackError::NoResolver(format!("no search provider for {}", source.as_str()))
        })?;
        provider.search(keyword, context).await
    }
}

impl Default for SearchRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 将 extractor Track 转为 PlayableEntry；ID 缺少来源前缀时跳过。
pub(crate) fn track_to_entry(track: Track, source: SourceKind) -> Option<PlayableEntry> {
    let remote_id = match source {
        SourceKind::Youtube => track.id.strip_prefix("yt:")?,
        SourceKind::Bilibili => track.id.strip_prefix("bili:")?,
        SourceKind::Wechat => return None,
        // ==== sync-generated:begin track_to_entry ====
        SourceKind::Audius => track.id.strip_prefix("au:")?,
        // ==== sync-generated:end track_to_entry ====
    };
    if remote_id.is_empty() {
        return None;
    }
    let source_ref = match source {
        SourceKind::Youtube => SourceRef::Youtube {
            video_id: remote_id.to_string(),
        },
        SourceKind::Bilibili => SourceRef::Bilibili {
            bvid: remote_id.to_string(),
        },
        SourceKind::Wechat => return None,
        // ==== sync-generated:begin track_to_entry_ref ====
        SourceKind::Audius => SourceRef::Audius {
            id: remote_id.to_string(),
        },
        // ==== sync-generated:end track_to_entry_ref ====
    };
    Some(PlayableEntry {
        view: track_to_view(track),
        source_ref,
    })
}

/// Convert extractor Track to the stable frontend TrackView.
fn track_to_view(track: Track) -> TrackView {
    let artist = track.artists.join(", ");
    let cover_url = track
        .artwork
        .first()
        .map(|img| img.url.clone())
        .unwrap_or_default();
    let duration_secs = track
        .duration_ms
        .map(|ms| ms as f32 / 1000.0)
        .unwrap_or(0.0);

    TrackView::new(track.title, artist, cover_url, duration_secs, track.id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_to_entry_skips_ids_without_source_prefix() {
        let track = Track {
            id: "plain-id".to_string(),
            title: "t".to_string(),
            artists: vec![],
            album: None,
            duration_ms: None,
            artwork: vec![],
        };
        assert!(track_to_entry(track, SourceKind::Youtube).is_none());
    }

    #[test]
    fn track_to_entry_maps_extractor_track() {
        let track = Track {
            id: "yt:video123".to_string(),
            title: "Title".to_string(),
            artists: vec!["Artist".to_string()],
            album: None,
            duration_ms: Some(90_000),
            artwork: vec![crate::extractor::model::Image {
                url: "https://img".to_string(),
                width: None,
                height: None,
            }],
        };
        let entry = track_to_entry(track, SourceKind::Youtube).expect("entry");
        assert_eq!(entry.view.id, "yt:video123");
        assert_eq!(entry.view.duration, 90.0);
        assert!(matches!(
            entry.source_ref,
            SourceRef::Youtube { video_id } if video_id == "video123"
        ));
    }
}
