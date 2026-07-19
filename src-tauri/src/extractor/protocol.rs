use crate::extractor::context::ExtractorContext;
use crate::extractor::model::RawMediaInfo;
use async_trait::async_trait;

/// Input fed to an extractor to determine what to extract.
#[derive(Debug, Clone)]
pub struct ExtractInput {
    /// URL to extract from (e.g. a YouTube watch URL or search URL).
    pub url: String,
    /// Optional extra parameters (e.g. search query for search extractors).
    pub extra: std::collections::HashMap<String, String>,
}

impl ExtractInput {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            extra: std::collections::HashMap::new(),
        }
    }

    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

/// Result types an extractor can return.
/// Mirrors yt-dlp's `_type` field semantics.
#[derive(Debug, Clone)]
pub enum ExtractorResult {
    /// A single media entry (video/audio track).
    Media(RawMediaInfo),
    /// A playlist of entries.
    Playlist(PlaylistInfo),
    /// Redirect to another URL for a different extractor to handle.
    Redirect(RedirectInfo),
    /// Transparent redirect — the current extractor's metadata is more
    /// precise; the resolved URL should incorporate it.
    TransparentRedirect(TransparentRedirectInfo),
    /// Multiple videos that form a single show (multi-video).
    MultiMedia(MultiMediaInfo),
}

#[derive(Debug, Clone)]
pub struct PlaylistInfo {
    pub id: Option<String>,
    pub title: Option<String>,
    pub entries: Vec<ExtractorResult>,
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct RedirectInfo {
    pub url: String,
    pub ie_key: Option<String>,
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct TransparentRedirectInfo {
    pub url: String,
    pub ie_key: Option<String>,
    /// Metadata that overrides or supplements the resolved URL's info.
    pub meta: RawMediaInfo,
}

#[derive(Debug, Clone)]
pub struct MultiMediaInfo {
    pub id: Option<String>,
    pub title: Option<String>,
    pub entries: Vec<ExtractorResult>,
    /// Common metadata shared by all entries.
    pub common_meta: Option<RawMediaInfo>,
}

/// Errors returned by extractors.
#[derive(Debug, thiserror::Error)]
pub enum ExtractError {
    #[error("unsupported URL: {0}")]
    Unsupported(String),

    #[error("extraction failed: {0}")]
    ExtractionFailed(String),

    #[error("network error: {0}")]
    NetworkError(String),

    #[error("parse error: {0}")]
    ParseError(String),

    #[error("rate limited")]
    RateLimited,

    #[error("geo restricted")]
    GeoRestricted,

    #[error("requires authentication")]
    RequiresAuth,

    #[error("cancelled")]
    Cancelled,

    #[error("max redirect depth exceeded")]
    MaxRedirectDepth,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<reqwest::Error> for ExtractError {
    fn from(e: reqwest::Error) -> Self {
        ExtractError::NetworkError(e.to_string())
    }
}

/// The core extractor trait.
///
/// Mirrors yt-dlp's `InfoExtractor` protocol but in Rust-native form.
/// Every extractor is `Send + Sync` so it can be used across threads.
#[async_trait]
pub trait Extractor: Send + Sync {
    /// Unique key identifying this extractor (e.g. "youtube:music").
    fn key(&self) -> &'static str;

    /// Priority for URL matching. Higher = tried first.
    fn priority(&self) -> i32 {
        0
    }

    /// Return true if this extractor can handle the given input.
    fn matches(&self, input: &ExtractInput) -> bool;

    /// One-time initialization (e.g. loading cookies, JS runtime).
    async fn initialize(&self, _context: &ExtractorContext) -> Result<(), ExtractError> {
        Ok(())
    }

    /// Main extraction method.
    async fn extract(
        &self,
        input: ExtractInput,
        context: &ExtractorContext,
    ) -> Result<ExtractorResult, ExtractError>;
}
