use async_trait::async_trait;

use crate::extractor::{bilibili, context::ExtractorContext, model::PlaybackManifest};

use super::{
    model::{PlayableEntry, SourceKind, SourceRef},
    resolver::{PlaybackError, PlaybackResolver},
    search::{track_to_entry, SearchProvider},
};

/// Bilibili 来源适配器：搜索 + 播放解析共用同一套来源抽象。
pub struct BilibiliSource;

#[async_trait]
impl PlaybackResolver for BilibiliSource {
    fn source(&self) -> SourceKind {
        SourceKind::Bilibili
    }

    async fn resolve(
        &self,
        entry: &PlayableEntry,
        context: &ExtractorContext,
    ) -> Result<PlaybackManifest, PlaybackError> {
        let SourceRef::Bilibili { bvid } = &entry.source_ref else {
            return Err(PlaybackError::NoResolver(
                SourceKind::Bilibili.as_str().to_string(),
            ));
        };
        bilibili::player::get_video_manifest(context, bvid)
            .await
            .map_err(Into::into)
    }
}

#[async_trait]
impl SearchProvider for BilibiliSource {
    fn source(&self) -> SourceKind {
        SourceKind::Bilibili
    }

    async fn search(
        &self,
        keyword: &str,
        context: &ExtractorContext,
    ) -> Result<Vec<PlayableEntry>, PlaybackError> {
        let tracks = bilibili::search::search_video(context, keyword, 1).await?;
        Ok(tracks
            .into_iter()
            .filter_map(|track| track_to_entry(track, SourceKind::Bilibili))
            .collect())
    }
}
