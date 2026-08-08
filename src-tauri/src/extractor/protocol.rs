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
