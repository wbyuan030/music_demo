use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;

use crate::extractor::{
    context::ExtractorContext, model::PlaybackManifest, protocol::ExtractError,
};

use super::model::{PlayableEntry, SourceKind};

#[derive(Debug, thiserror::Error)]
pub enum PlaybackError {
    #[error("track not found: {0}")]
    TrackNotFound(String),
    #[error("no resolver for source: {0}")]
    NoResolver(String),
    #[error("no playable audio stream: {0}")]
    NoPlayableStream(String),
    #[error("source extraction failed: {0}")]
    Extraction(#[from] ExtractError),
    #[error("network error: {0}")]
    Network(String),
    #[error("decode error: {0}")]
    Decode(String),
    #[error("cancelled")]
    Cancelled,
}

#[async_trait]
pub trait PlaybackResolver: Send + Sync {
    fn source(&self) -> SourceKind;

    fn accepts(&self, entry: &PlayableEntry) -> bool {
        entry.source_ref.kind() == self.source()
    }

    async fn resolve(
        &self,
        entry: &PlayableEntry,
        context: &ExtractorContext,
    ) -> Result<PlaybackManifest, PlaybackError>;
}

pub struct ResolverRegistry {
    resolvers: HashMap<SourceKind, Vec<Arc<dyn PlaybackResolver>>>,
}

impl ResolverRegistry {
    pub fn new() -> Self {
        Self {
            resolvers: HashMap::new(),
        }
    }

    pub fn register<R>(&mut self, resolver: R)
    where
        R: PlaybackResolver + 'static,
    {
        self.resolvers
            .entry(resolver.source())
            .or_default()
            .push(Arc::new(resolver));
    }

    pub async fn resolve(
        &self,
        entry: &PlayableEntry,
        context: &ExtractorContext,
    ) -> Result<PlaybackManifest, PlaybackError> {
        let source = entry.source_ref.kind();
        let resolvers = self
            .resolvers
            .get(&source)
            .ok_or_else(|| PlaybackError::NoResolver(source.as_str().to_string()))?;

        for resolver in resolvers {
            if resolver.accepts(entry) {
                return resolver.resolve(entry, context).await;
            }
        }

        Err(PlaybackError::NoResolver(source.as_str().to_string()))
    }
}

impl Default for ResolverRegistry {
    fn default() -> Self {
        Self::new()
    }
}
