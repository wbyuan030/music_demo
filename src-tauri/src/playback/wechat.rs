use std::collections::HashMap;

use async_trait::async_trait;

use crate::extractor::{
    context::ExtractorContext,
    model::{AudioStream, PlaybackManifest},
};

use super::{
    model::{PlayableEntry, SourceKind, SourceRef},
    resolver::{PlaybackError, PlaybackResolver},
};

pub struct WechatResolver;

fn manifest_for_url(url: &str, user_agent: &str) -> PlaybackManifest {
    let mut headers = HashMap::new();
    headers.insert("User-Agent".to_string(), user_agent.to_string());
    headers.insert("Accept-Language".to_string(), "zh-CN,zh;q=0.9".to_string());
    headers.insert("Referer".to_string(), url.to_string());
    PlaybackManifest {
        streams: vec![AudioStream {
            url: url.to_string(),
            mime_type: "audio/mpeg".to_string(),
            bitrate: None,
            codec: None,
            content_length: None,
        }],
        headers,
        expires_at: None,
    }
}

#[async_trait]
impl PlaybackResolver for WechatResolver {
    fn source(&self) -> SourceKind {
        SourceKind::Wechat
    }

    fn accepts(&self, entry: &PlayableEntry) -> bool {
        matches!(entry.source_ref, SourceRef::Wechat { .. })
    }

    async fn resolve(
        &self,
        entry: &PlayableEntry,
        context: &ExtractorContext,
    ) -> Result<PlaybackManifest, PlaybackError> {
        let url = match &entry.source_ref {
            SourceRef::Wechat { url } => url.clone(),
            _ => {
                return Err(PlaybackError::NoResolver(
                    SourceKind::Wechat.as_str().to_string(),
                ))
            }
        };
        if url.is_empty() {
            return Err(PlaybackError::NoPlayableStream(entry.id().to_string()));
        }
        Ok(manifest_for_url(&url, &context.options.user_agent))
    }
}
