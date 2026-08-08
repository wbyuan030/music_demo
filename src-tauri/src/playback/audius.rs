use async_trait::async_trait;

use crate::extractor::{context::ExtractorContext, model::PlaybackManifest};

use super::{
    model::{PlayableEntry, SourceKind, SourceRef},
    resolver::{PlaybackError, PlaybackResolver},
    search::{track_to_entry, SearchProvider},
};

/// Audius 来源适配器：搜索 + 播放解析共用同一套来源抽象。
pub struct AudiusSource;

#[async_trait]
impl SearchProvider for AudiusSource {
    fn source(&self) -> SourceKind {
        SourceKind::Audius
    }

    async fn search(
        &self,
        keyword: &str,
        context: &ExtractorContext,
    ) -> Result<Vec<PlayableEntry>, PlaybackError> {
        let tracks = crate::extractor::audius::search::search_music(context, keyword, None).await?;
        Ok(tracks
            .into_iter()
            .filter_map(|track| track_to_entry(track, SourceKind::Audius))
            .collect())
    }
}

#[async_trait]
impl PlaybackResolver for AudiusSource {
    fn source(&self) -> SourceKind {
        SourceKind::Audius
    }

    async fn resolve(
        &self,
        entry: &PlayableEntry,
        context: &ExtractorContext,
    ) -> Result<PlaybackManifest, PlaybackError> {
        let SourceRef::Audius { id } = &entry.source_ref else {
            return Err(PlaybackError::NoResolver(
                SourceKind::Audius.as_str().to_string(),
            ));
        };
        crate::extractor::audius::player::get_manifest(context, id)
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractor::model::Track;

    #[test]
    fn track_id_prefix_is_stable() {
        let id = format!("au:abc123");
        let track = Track {
            id: id.clone(),
            title: "t".into(),
            artists: vec![],
            album: None,
            duration_ms: None,
            artwork: vec![],
        };
        let entry = track_to_entry(track, SourceKind::Audius).expect("entry");
        assert_eq!(entry.view.id, id);
    }

    #[test]
    fn resolver_rejects_wrong_source() {
        let entry = PlayableEntry {
            view: crate::types::TrackView::new(
                "x".into(),
                "y".into(),
                "".into(),
                0.0,
                "yt:abc".into(),
            ),
            source_ref: SourceRef::Youtube {
                video_id: "abc".into(),
            },
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let ctx = ExtractorContext::with_client(reqwest::Client::new());
            AudiusSource.resolve(&entry, &ctx).await
        });
        assert!(matches!(result, Err(PlaybackError::NoResolver(_))));
    }

    /// 行为测试 3：resolve 能拿到流 → PlaybackService 流式下载
    /// 到临时文件 → 内容一致 → 二次加载命中缓存。
    #[tokio::test(flavor = "multi_thread")]
    async fn resolves_then_spools_and_caches() {
        use std::sync::Arc;

        use crate::extractor::context::ExtractorOptions;
        use crate::global::init_db_at;
        use crate::playback::catalog::TrackCatalog;
        use crate::playback::resolver::ResolverRegistry;
        use crate::playback::service::{PlaybackService, TrackSource};
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio_util::sync::CancellationToken;

        static DB_INIT: std::sync::Once = std::sync::Once::new();

        DB_INIT.call_once(|| {
            let _ = init_db_at(
                &std::env::temp_dir()
                    .join(format!("music-audius-test-{}.db", uuid::Uuid::new_v4()))
                    .to_string_lossy()
                    .into_owned(),
            );
        });

        // 音频 mock：构造一段合法 WAV（与服务.rs 测试同构）
        let data_len = 9_600_u32;
        let mut audio = Vec::with_capacity(44 + data_len as usize);
        audio.extend_from_slice(b"RIFF");
        audio.extend_from_slice(&(36 + data_len).to_le_bytes());
        audio.extend_from_slice(b"WAVEfmt ");
        audio.extend_from_slice(&16_u32.to_le_bytes());
        audio.extend_from_slice(&1_u16.to_le_bytes());
        audio.extend_from_slice(&1_u16.to_le_bytes());
        audio.extend_from_slice(&48_000_u32.to_le_bytes());
        audio.extend_from_slice(&96_000_u32.to_le_bytes());
        audio.extend_from_slice(&2_u16.to_le_bytes());
        audio.extend_from_slice(&16_u16.to_le_bytes());
        audio.extend_from_slice(b"data");
        audio.extend_from_slice(&data_len.to_le_bytes());
        audio.resize(44 + data_len as usize, 0);
        let expected = audio.clone();
        // 单 listener 按序服务两个请求：第一个是 API（resolve 用），第二个是音频（spool 用）。
        // （不能两个 listener 各自 accept——同机连接会被先 spawn 的 accept 抢走，造成死锁）
        let raw = include_str!("../extractor/audius/fixtures/player.json");
        assert_ne!(raw.trim(), "{}", "fixtures/player.json 未录制");
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let addr = listener.local_addr().unwrap();
        let mut fixture: serde_json::Value = serde_json::from_str(raw).unwrap();
        fixture["data"]["stream"]["url"] =
            serde_json::Value::String(format!("http://{}/audio", addr));
        let body = fixture.to_string();

        let server = tokio::spawn(async move {
            // 第一个请求：API（resolve 用）
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = [0u8; 4096];
            let _ = socket.read(&mut request).await;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            socket.write_all(resp.as_bytes()).await.unwrap();
            // 第二个请求：音频（spool 下载用）
            let (mut socket2, _) = listener.accept().await.unwrap();
            let mut request2 = [0u8; 4096];
            let _ = socket2.read(&mut request2).await;
            let mut response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                audio.len()
            )
            .into_bytes();
            response.extend_from_slice(&audio);
            socket2.write_all(&response).await.unwrap();
            let _ = socket2.shutdown().await;
        });

        let context = Arc::new(
            ExtractorContext::with_client(reqwest::Client::builder().no_proxy().build().unwrap())
                .with_options(
                    ExtractorOptions::default()
                        .with_endpoint("player", &format!("http://{}", addr)),
                ),
        );
        let catalog = Arc::new(TrackCatalog::new());
        let mut registry = ResolverRegistry::new();
        registry.register(AudiusSource);
        let cache_dir =
            std::env::temp_dir().join(format!("music-audius-cache-{}", uuid::Uuid::new_v4()));
        let service = PlaybackService::new(
            Arc::clone(&context),
            Arc::clone(&catalog),
            Arc::new(registry),
            cache_dir.clone(),
        );
        let entry = PlayableEntry {
            view: crate::types::TrackView::new(
                "Got That Drip".to_string(),
                "Drippies".to_string(),
                String::new(),
                96.0,
                "au:jaKgV".to_string(),
            ),
            source_ref: SourceRef::Audius {
                id: "jaKgV".to_string(),
            },
        };
        catalog.insert(entry.view.id.clone(), entry).await;

        let cancel = CancellationToken::new();
        let source = service
            .load_track_source("au:jaKgV", &cancel)
            .await
            .unwrap();
        let (state, stable_path) = match source {
            TrackSource::Progressive {
                reader,
                state,
                path,
            } => {
                let mut reader = reader;
                let mut buf = Vec::new();
                std::io::Read::read_to_end(&mut reader, &mut buf).unwrap();
                assert_eq!(buf, expected);
                state.mark_decoded();
                (state, path)
            }
            TrackSource::File(_) => panic!("expected progressive source"),
        };
        server.await.unwrap();

        for _ in 0..100 {
            if tokio::fs::try_exists(&stable_path).await.unwrap() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        assert_eq!(tokio::fs::read(&stable_path).await.unwrap(), expected);
        assert_eq!(state.failure(), None);

        let cached = service
            .load_track_source("au:jaKgV", &cancel)
            .await
            .unwrap();
        match cached {
            TrackSource::File(path) => assert_eq!(path, stable_path),
            TrackSource::Progressive { .. } => panic!("expected cached file source"),
        }

        let _ = tokio::fs::remove_dir_all(&cache_dir).await;
    }
}
