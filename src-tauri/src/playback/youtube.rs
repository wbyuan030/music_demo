use async_trait::async_trait;

use crate::extractor::{context::ExtractorContext, model::PlaybackManifest, youtube};

use super::{
    model::{PlayableEntry, SourceKind, SourceRef},
    resolver::{PlaybackError, PlaybackResolver},
    search::{track_to_entry, SearchProvider},
};

/// YouTube 来源适配器：搜索 + 播放解析共用同一套来源抽象。
pub struct YoutubeSource;

#[async_trait]
impl PlaybackResolver for YoutubeSource {
    fn source(&self) -> SourceKind {
        SourceKind::Youtube
    }

    async fn resolve(
        &self,
        entry: &PlayableEntry,
        context: &ExtractorContext,
    ) -> Result<PlaybackManifest, PlaybackError> {
        let SourceRef::Youtube { video_id } = &entry.source_ref else {
            return Err(PlaybackError::NoResolver(
                SourceKind::Youtube.as_str().to_string(),
            ));
        };
        youtube::player::get_manifest(context, video_id)
            .await
            .map_err(Into::into)
    }
}

#[async_trait]
impl SearchProvider for YoutubeSource {
    fn source(&self) -> SourceKind {
        SourceKind::Youtube
    }

    async fn search(
        &self,
        keyword: &str,
        context: &ExtractorContext,
    ) -> Result<Vec<PlayableEntry>, PlaybackError> {
        let tracks = youtube::search::search_music(context, keyword, Some("songs")).await?;
        Ok(tracks
            .into_iter()
            .filter_map(|track| track_to_entry(track, SourceKind::Youtube))
            .collect())
    }
}
