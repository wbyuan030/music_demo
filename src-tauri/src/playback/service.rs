use std::{
    cmp::Reverse,
    fs::File,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Instant, SystemTime},
};

use rodio::Decoder;
use tokio_util::sync::CancellationToken;

use crate::{
    extractor::{context::ExtractorContext, model::PlaybackManifest},
    global::get_db,
    storage::{_remove_recent_track, add_recent_track, get_track_by_id, upsert_track_entry},
};

use super::{
    catalog::TrackCatalog,
    model::TrackId,
    resolver::{PlaybackError, ResolverRegistry},
    spool::{BlockingSpoolReader, SpoolState},
    trace::PlaybackTrace,
};

/// 播放来源：完整缓存文件，或边下边播的 spool。
pub enum TrackSource {
    /// 已落盘的完整缓存文件。
    File(PathBuf),
    /// 边下边播；`state` 用于标记解码成功/查询失败，下载完成后文件落到 `path`。
    Progressive {
        reader: BlockingSpoolReader,
        state: Arc<SpoolState>,
        path: PathBuf,
    },
}

pub struct PlaybackService {
    pub(crate) context: Arc<ExtractorContext>,
    pub(crate) catalog: Arc<TrackCatalog>,
    pub(crate) resolvers: Arc<ResolverRegistry>,
    pub(crate) cache_dir: PathBuf,
}

impl PlaybackService {
    pub fn new(
        context: Arc<ExtractorContext>,
        catalog: Arc<TrackCatalog>,
        resolvers: Arc<ResolverRegistry>,
        cache_dir: PathBuf,
    ) -> Self {
        Self {
            context,
            catalog,
            resolvers,
            cache_dir,
        }
    }

    pub async fn load_track_source(
        &self,
        id: &str,
        cancel: &CancellationToken,
    ) -> Result<TrackSource, PlaybackError> {
        let trace = PlaybackTrace::new(id);
        trace.event("load", "status=start");
        self.load_track_source_with_trace(id, cancel, &trace).await
    }

    pub(crate) async fn load_track_source_with_trace(
        &self,
        id: &str,
        cancel: &CancellationToken,
        trace: &PlaybackTrace,
    ) -> Result<TrackSource, PlaybackError> {
        check_cancelled(cancel)?;

        let db_started = Instant::now();
        let db_item = match get_track_by_id(get_db(), id.to_string()) {
            Ok(item) => {
                trace.event(
                    "db_lookup",
                    format_args!(
                        "status=ok hit={} elapsed_ms={}",
                        item.is_some(),
                        db_started.elapsed().as_millis()
                    ),
                );
                item
            }
            Err(error) => {
                trace.event(
                    "db_lookup",
                    format_args!(
                        "status=error elapsed_ms={}",
                        db_started.elapsed().as_millis()
                    ),
                );
                return Err(PlaybackError::Network(format!(
                    "database lookup failed: {}",
                    error
                )));
            }
        };
        let entry = match self.catalog.get(id).await {
            Some(entry) => entry,
            None => db_item
                .as_ref()
                .ok_or_else(|| PlaybackError::TrackNotFound(id.to_string()))?
                .to_playable_entry()
                .map_err(|e| PlaybackError::TrackNotFound(e.to_string()))?,
        };
        trace.event(
            "entry",
            format_args!("status=ok source={}", entry.source_ref.kind().as_str()),
        );

        if let Some(item) = &db_item {
            if !item.src.is_empty() {
                let path = PathBuf::from(&item.src);
                let status = inspect_cache(&path, cancel).await?;
                trace.event(
                    "cache",
                    format_args!(
                        "location=db status={} bytes={}",
                        status.as_str(),
                        status.bytes()
                    ),
                );
                if status.is_valid() {
                    return Ok(TrackSource::File(path));
                }
            }
        }

        let stable_path = self.cache_path(id);
        let status = inspect_cache(&stable_path, cancel).await?;
        trace.event(
            "cache",
            format_args!(
                "location=stable status={} bytes={}",
                status.as_str(),
                status.bytes()
            ),
        );
        if status.is_valid() {
            return Ok(TrackSource::File(stable_path));
        }

        check_cancelled(cancel)?;
        let resolver_started = Instant::now();
        trace.event(
            "resolver",
            format_args!("status=start source={}", entry.source_ref.kind().as_str()),
        );
        let resolved = tokio::select! {
            _ = cancel.cancelled() => return Err(PlaybackError::Cancelled),
            result = self.resolvers.resolve(&entry, &self.context) => result,
        };
        let mut manifest = match resolved {
            Ok(manifest) => manifest,
            Err(error) => {
                trace.event(
                    "resolver",
                    format_args!(
                        "status=error elapsed_ms={} error={}",
                        resolver_started.elapsed().as_millis(),
                        error
                    ),
                );
                return Err(error);
            }
        };
        trace.event(
            "resolver",
            format_args!(
                "status=ok streams={} headers={} elapsed_ms={} expires_in_sec={}",
                manifest.streams.len(),
                manifest.headers.len(),
                resolver_started.elapsed().as_millis(),
                expires_in_seconds(&manifest)
            ),
        );
        if manifest
            .expires_at
            .is_some_and(|expires_at| SystemTime::now() >= expires_at)
        {
            trace.event("resolver", "status=expired_refresh");
            check_cancelled(cancel)?;
            let refresh_started = Instant::now();
            let refreshed = tokio::select! {
                _ = cancel.cancelled() => return Err(PlaybackError::Cancelled),
                result = self.resolvers.resolve(&entry, &self.context) => result,
            };
            manifest = match refreshed {
                Ok(manifest) => manifest,
                Err(error) => {
                    trace.event(
                        "resolver",
                        format_args!(
                            "status=refresh_error elapsed_ms={} error={}",
                            refresh_started.elapsed().as_millis(),
                            error
                        ),
                    );
                    return Err(error);
                }
            };
            trace.event(
                "resolver",
                format_args!(
                    "status=refresh_ok streams={} elapsed_ms={} expires_in_sec={}",
                    manifest.streams.len(),
                    refresh_started.elapsed().as_millis(),
                    expires_in_seconds(&manifest)
                ),
            );
        }
        if manifest
            .expires_at
            .is_some_and(|expires_at| SystemTime::now() >= expires_at)
        {
            trace.event("resolver", "status=expired_after_refresh");
            return Err(PlaybackError::Extraction(
                crate::extractor::protocol::ExtractError::ExtractionFailed(
                    "resolved manifest is expired".to_string(),
                ),
            ));
        }

        self.start_streaming_download(manifest, &stable_path, cancel, trace)
            .await
    }

    pub async fn persist_played(&self, id: &str, cached_path: &Path) -> Result<(), PlaybackError> {
        let entry = match self.catalog.get(id).await {
            Some(entry) => entry,
            None => get_track_by_id(get_db(), id.to_string())
                .map_err(|e| PlaybackError::Network(format!("database lookup failed: {}", e)))?
                .ok_or_else(|| PlaybackError::TrackNotFound(id.to_string()))?
                .to_playable_entry()
                .map_err(|e| PlaybackError::TrackNotFound(e.to_string()))?,
        };
        let item = upsert_track_entry(get_db(), &entry, Some(cached_path))
            .map_err(|e| PlaybackError::Network(format!("persist played track failed: {}", e)))?;
        add_recent_track(get_db(), item)
            .map_err(|e| PlaybackError::Network(format!("persist recent track failed: {}", e)))?;
        Ok(())
    }
    pub async fn discard_played(&self, id: &str, cached_path: &Path) -> Result<(), PlaybackError> {
        _remove_recent_track(get_db(), id.to_string())
            .map_err(|e| PlaybackError::Network(format!("discard recent track failed: {}", e)))?;
        match tokio::fs::remove_file(cached_path).await {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(PlaybackError::Network(format!(
                "discard cache failed: {}",
                error
            ))),
        }
    }

    fn cache_path(&self, id: &str) -> PathBuf {
        let stable_id = id
            .parse::<TrackId>()
            .map(|track_id| track_id.to_string())
            .unwrap_or_else(|_| id.to_string());
        let hash = format!("{:x}", md5::compute(stable_id.as_bytes()));
        self.cache_dir.join(format!("{}.audio", hash))
    }

    /// 流式下载：边写临时文件边让解码器读（spool），下载完成且解码成功后原子提交缓存。
    async fn start_streaming_download(
        &self,
        manifest: PlaybackManifest,
        stable_path: &Path,
        cancel: &CancellationToken,
        trace: &PlaybackTrace,
    ) -> Result<TrackSource, PlaybackError> {
        if manifest.streams.is_empty() {
            trace.event("download", "status=error reason=no_streams");
            return Err(PlaybackError::NoPlayableStream(
                stable_path.to_string_lossy().into_owned(),
            ));
        }

        let mut streams = manifest.streams;
        streams.sort_by_key(|stream| Reverse(stream.bitrate.unwrap_or(0)));
        let total_streams = streams.len();
        let mut last_error = None;

        let create_dir = tokio::select! {
            _ = cancel.cancelled() => return Err(PlaybackError::Cancelled),
            result = tokio::fs::create_dir_all(&self.cache_dir) => result,
        };
        if let Err(error) = create_dir {
            trace.event(
                "cache_directory",
                format_args!("status=error error={}", error),
            );
            return Err(PlaybackError::Network(format!(
                "create cache directory failed: {}",
                error
            )));
        }
        trace.event(
            "download",
            format_args!("status=start candidates={}", total_streams),
        );

        let stable_name = stable_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("track");

        for (index, stream) in streams.into_iter().enumerate() {
            if let Err(error) = check_cancelled(cancel) {
                return Err(error);
            }
            let candidate = index + 1;
            let stream_hash = url_fingerprint(&stream.url);
            trace.event(
                "download_candidate",
                format_args!(
                    "status=start candidate={}/{} mime={} codec={} bitrate={} expected_bytes={} url_hash={}",
                    candidate,
                    total_streams,
                    stream.mime_type,
                    stream.codec.as_deref().unwrap_or("unknown"),
                    stream
                        .bitrate
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "unknown".to_string()),
                    stream
                        .content_length
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "unknown".to_string()),
                    stream_hash
                ),
            );
            if stream.url.is_empty() {
                last_error = Some("empty audio stream URL".to_string());
                trace.event(
                    "download_candidate",
                    format_args!("status=error candidate={} reason=empty_url", candidate),
                );
                continue;
            }

            let request_started = Instant::now();
            let request = self
                .context
                .http
                .get(&stream.url)
                .headers(to_reqwest_headers(&manifest.headers));
            let response = tokio::select! {
                _ = cancel.cancelled() => return Err(PlaybackError::Cancelled),
                result = request.send() => result.map_err(|error| error.to_string()),
            };
            let response = match response {
                Ok(response) if response.status().is_success() => {
                    trace.event(
                        "http",
                        format_args!(
                            "status={} candidate={} headers_elapsed_ms={} content_length={}",
                            response.status().as_u16(),
                            candidate,
                            request_started.elapsed().as_millis(),
                            response
                                .content_length()
                                .map(|value| value.to_string())
                                .unwrap_or_else(|| "unknown".to_string())
                        ),
                    );
                    response
                }
                Ok(response) => {
                    let status = response.status();
                    last_error = Some(format!("HTTP {} for stream_hash={}", status, stream_hash));
                    trace.event(
                        "http",
                        format_args!(
                            "status=error candidate={} http_status={} elapsed_ms={}",
                            candidate,
                            status.as_u16(),
                            request_started.elapsed().as_millis()
                        ),
                    );
                    continue;
                }
                Err(error) => {
                    last_error = Some(error.clone());
                    trace.event(
                        "http",
                        format_args!(
                            "status=error candidate={} elapsed_ms={} error={}",
                            candidate,
                            request_started.elapsed().as_millis(),
                            error
                        ),
                    );
                    continue;
                }
            };

            // 首个成功的候选流：创建 spool，立即返回可读句柄，后台任务继续写文件。
            let temporary_path =
                self.cache_dir
                    .join(format!(".{}.{}.tmp", stable_name, uuid::Uuid::new_v4()));
            let state = Arc::new(SpoolState::new());
            if let Some(total) = response.content_length() {
                *state.total.lock() = Some(total);
            }
            let writer_file = match tokio::fs::File::create(&temporary_path).await {
                Ok(file) => file,
                Err(error) => {
                    last_error = Some(format!("create spool failed: {}", error));
                    trace.event(
                        "cache_write",
                        format_args!("status=error candidate={} error={}", candidate, error),
                    );
                    continue;
                }
            };
            let reader_file = match std::fs::File::open(&temporary_path) {
                Ok(file) => file,
                Err(error) => {
                    let _ = tokio::fs::remove_file(&temporary_path).await;
                    last_error = Some(format!("open spool failed: {}", error));
                    trace.event(
                        "cache_write",
                        format_args!("status=error candidate={} error={}", candidate, error),
                    );
                    continue;
                }
            };

            self.spawn_stream_writer(
                response,
                writer_file,
                temporary_path,
                stable_path.to_path_buf(),
                cancel.clone(),
                trace.clone(),
                Arc::clone(&state),
            );

            trace.event(
                "stream",
                format_args!(
                    "status=started candidate={} spool={}",
                    candidate,
                    state.downloaded.load(std::sync::atomic::Ordering::Relaxed)
                ),
            );
            return Ok(TrackSource::Progressive {
                reader: BlockingSpoolReader::new(reader_file, Arc::clone(&state)),
                state,
                path: stable_path.to_path_buf(),
            });
        }

        trace.event(
            "download",
            format_args!(
                "status=error reason=all_candidates_failed last_error={}",
                last_error.as_deref().unwrap_or("unknown")
            ),
        );
        Err(PlaybackError::Network(
            last_error.unwrap_or_else(|| "all audio streams failed".to_string()),
        ))
    }

    /// 后台下载任务：逐块写 spool 文件；下载成功后等解码器打开，再原子提交缓存。
    fn spawn_stream_writer(
        &self,
        response: reqwest::Response,
        writer: tokio::fs::File,
        temporary_path: PathBuf,
        stable_path: PathBuf,
        cancel: CancellationToken,
        trace: PlaybackTrace,
        state: Arc<SpoolState>,
    ) -> tokio::task::JoinHandle<()> {
        let mut writer = writer;
        tokio::spawn(async move {
            use futures::StreamExt as _;
            use tokio::io::AsyncWriteExt as _;

            let body_started = Instant::now();
            let mut stream = response.bytes_stream();
            let mut total_written: u64 = 0;
            loop {
                let chunk = tokio::select! {
                    _ = cancel.cancelled() => break,
                    result = stream.next() => result,
                };
                let chunk = match chunk {
                    Some(Ok(chunk)) => chunk,
                    Some(Err(error)) => {
                        let _ = tokio::fs::remove_file(&temporary_path).await;
                        trace.event(
                            "http_body",
                            format_args!("status=error reason=stream_failed error={}", error),
                        );
                        state.finish(Some(format!("stream failed: {}", error)));
                        return;
                    }
                    None => break,
                };
                if let Err(error) = writer.write_all(&chunk).await {
                    let _ = tokio::fs::remove_file(&temporary_path).await;
                    trace.event("cache_write", format_args!("status=error error={}", error));
                    state.finish(Some(format!("cache write failed: {}", error)));
                    return;
                }
                total_written += chunk.len() as u64;
                state.add_downloaded(chunk.len() as u64);
            }

            if cancel.is_cancelled() {
                let _ = tokio::fs::remove_file(&temporary_path).await;
                trace.event("download", "status=cancelled");
                state.finish(Some("cancelled".to_string()));
                return;
            }

            trace.event(
                "http_body",
                format_args!(
                    "status=ok bytes={} elapsed_ms={}",
                    total_written,
                    body_started.elapsed().as_millis()
                ),
            );
            // 下载完成：数据全部可用，读者可自由读到 EOF。
            state.finish(None);

            if !state.wait_decoded(&cancel).await {
                let _ = tokio::fs::remove_file(&temporary_path).await;
                trace.event("cache_commit", "status=cancelled_before_commit");
                return;
            }
            if state.decode_failed() {
                let _ = tokio::fs::remove_file(&temporary_path).await;
                trace.event("cache_commit", "status=aborted_decode_failed");
                return;
            }

            if let Err(error) = tokio::fs::rename(&temporary_path, &stable_path).await {
                let _ = tokio::fs::remove_file(&temporary_path).await;
                trace.event("cache_commit", format_args!("status=error error={}", error));
                return;
            }
            trace.event(
                "cache_commit",
                format_args!(
                    "status=ok bytes={} path={}",
                    total_written,
                    stable_path.display()
                ),
            );
        })
    }
}

fn check_cancelled(cancel: &CancellationToken) -> Result<(), PlaybackError> {
    if cancel.is_cancelled() {
        Err(PlaybackError::Cancelled)
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CacheStatus {
    Missing,
    Invalid { bytes: u64 },
    Valid { bytes: u64 },
}

impl CacheStatus {
    fn is_valid(self) -> bool {
        matches!(self, Self::Valid { .. })
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Missing => "miss",
            Self::Invalid { .. } => "invalid",
            Self::Valid { .. } => "hit",
        }
    }

    fn bytes(self) -> u64 {
        match self {
            Self::Missing => 0,
            Self::Invalid { bytes } | Self::Valid { bytes } => bytes,
        }
    }
}

async fn inspect_cache(
    path: &Path,
    cancel: &CancellationToken,
) -> Result<CacheStatus, PlaybackError> {
    let metadata = tokio::select! {
        _ = cancel.cancelled() => return Err(PlaybackError::Cancelled),
        result = tokio::fs::metadata(path) => match result {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(CacheStatus::Missing);
            }
            Err(_) => return Ok(CacheStatus::Invalid { bytes: 0 }),
        },
    };
    let bytes = metadata.len();
    if !metadata.is_file() || bytes == 0 {
        return Ok(CacheStatus::Invalid { bytes });
    }

    check_cancelled(cancel)?;
    let path = path.to_path_buf();
    let valid = tokio::select! {
        _ = cancel.cancelled() => return Err(PlaybackError::Cancelled),
        result = tokio::task::spawn_blocking(move || {
            File::open(path)
                .ok()
                .and_then(|file| Decoder::try_from(file).ok())
                .is_some()
        }) => result.unwrap_or(false),
    };
    Ok(if valid {
        CacheStatus::Valid { bytes }
    } else {
        CacheStatus::Invalid { bytes }
    })
}

#[cfg(test)]
async fn readable_file(path: &Path, cancel: &CancellationToken) -> Result<bool, PlaybackError> {
    Ok(inspect_cache(path, cancel).await?.is_valid())
}

fn expires_in_seconds(manifest: &PlaybackManifest) -> u64 {
    manifest
        .expires_at
        .and_then(|expires_at| expires_at.duration_since(SystemTime::now()).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn url_fingerprint(url: &str) -> String {
    format!("{:x}", md5::compute(url.as_bytes()))
}

fn to_reqwest_headers(
    headers: &std::collections::HashMap<String, String>,
) -> reqwest::header::HeaderMap {
    headers
        .iter()
        .filter_map(|(name, value)| {
            Some((
                reqwest::header::HeaderName::try_from(name).ok()?,
                reqwest::header::HeaderValue::try_from(value).ok()?,
            ))
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

    use tokio::io::AsyncWriteExt;
    use tokio_util::sync::CancellationToken;

    use super::*;
    use crate::{
        extractor::context::ExtractorContext,
        global::init_db_at,
        playback::{
            catalog::TrackCatalog,
            model::{PlayableEntry, SourceRef},
            resolver::ResolverRegistry,
            wechat::WechatResolver,
        },
        types::TrackView,
    };
    static DB_INIT: std::sync::Once = std::sync::Once::new();

    // 独立临时 DB 文件：避开正在运行的桌面应用对 `local.db` 的 redb 文件锁。
    fn test_db_path() -> String {
        std::env::temp_dir()
            .join(format!("music-test-db-{}.db", std::process::id()))
            .to_string_lossy()
            .into_owned()
    }
    fn test_wav() -> Vec<u8> {
        let sample_count = 4_800_u32;
        let data_len = sample_count * 2;
        let mut wav = Vec::with_capacity(44 + data_len as usize);
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + data_len).to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16_u32.to_le_bytes());
        wav.extend_from_slice(&1_u16.to_le_bytes());
        wav.extend_from_slice(&1_u16.to_le_bytes());
        wav.extend_from_slice(&48_000_u32.to_le_bytes());
        wav.extend_from_slice(&96_000_u32.to_le_bytes());
        wav.extend_from_slice(&2_u16.to_le_bytes());
        wav.extend_from_slice(&16_u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_len.to_le_bytes());
        wav.resize(44 + data_len as usize, 0);
        wav
    }

    #[tokio::test]
    async fn rejects_non_decodable_cache_file() {
        let path =
            std::env::temp_dir().join(format!("music-invalid-cache-{}", uuid::Uuid::new_v4()));
        tokio::fs::write(&path, b"not-audio").await.unwrap();
        assert!(!readable_file(&path, &CancellationToken::new())
            .await
            .unwrap());
        tokio::fs::remove_file(path).await.unwrap();
    }

    // 测试会同步阻塞读 spool（模拟解码线程），需要多线程运行时让 writer 任务继续。
    #[tokio::test(flavor = "multi_thread")]
    async fn downloads_once_and_reuses_stable_cache() {
        DB_INIT.call_once(|| {
            let _ = init_db_at(&test_db_path());
        });
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .unwrap();
        let address = listener.local_addr().unwrap();
        let audio = test_wav();
        let expected = audio.clone();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            // 先读完请求再响应，避免关闭时有未读数据触发 RST。
            let mut request = [0u8; 4096];
            let _ = socket.read(&mut request).await;
            let mut response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                audio.len()
            )
            .into_bytes();
            response.extend_from_slice(&audio);
            socket.write_all(&response).await.unwrap();
            let _ = socket.shutdown().await;
        });

        let context = Arc::new(ExtractorContext::with_client(
            reqwest::Client::builder().no_proxy().build().unwrap(),
        ));
        let catalog = Arc::new(TrackCatalog::new());
        let mut registry = ResolverRegistry::new();
        registry.register(WechatResolver);
        let service = PlaybackService::new(
            context,
            Arc::clone(&catalog),
            Arc::new(registry),
            std::env::temp_dir().join(format!("music-playback-test-{}", uuid::Uuid::new_v4())),
        );
        let entry = PlayableEntry {
            view: TrackView::new(
                "title".to_string(),
                "artist".to_string(),
                String::new(),
                1.0,
                "legacy-test-id".to_string(),
            ),
            source_ref: SourceRef::Wechat {
                url: format!("http://{}/audio", address),
            },
        };
        catalog.insert(entry.view.id.clone(), entry).await;

        let cancel = CancellationToken::new();
        let source = service
            .load_track_source("legacy-test-id", &cancel)
            .await
            .unwrap();
        let (state, stable_path) = match source {
            TrackSource::Progressive {
                reader,
                state,
                path,
            } => {
                // 模拟解码器：把 spool 读到底（阻塞直到下载完成）
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

        // 下载完成 + 解码成功后，writer 原子提交缓存文件。
        for _ in 0..100 {
            if tokio::fs::try_exists(&stable_path).await.unwrap() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        assert_eq!(tokio::fs::read(&stable_path).await.unwrap(), expected);
        assert_eq!(state.failure(), None);

        let cached_source = service
            .load_track_source("legacy-test-id", &cancel)
            .await
            .unwrap();
        match cached_source {
            TrackSource::File(path) => assert_eq!(path, stable_path),
            TrackSource::Progressive { .. } => panic!("expected cached file source"),
        }

        let _ = tokio::fs::remove_dir_all(&service.cache_dir).await;
    }
    #[tokio::test]
    async fn cancellation_leaves_no_final_cache_file() {
        DB_INIT.call_once(|| {
            let _ = init_db_at(&test_db_path());
        });
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = [0u8; 4096];
            let _ = socket.read(&mut request).await;
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            let _ = socket
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 11\r\nConnection: close\r\n\r\ncache-bytes",
                )
                .await;
            let _ = socket.shutdown().await;
        });

        let context = Arc::new(ExtractorContext::with_client(
            reqwest::Client::builder().no_proxy().build().unwrap(),
        ));
        let catalog = Arc::new(TrackCatalog::new());
        let mut registry = ResolverRegistry::new();
        registry.register(WechatResolver);
        let cache_dir =
            std::env::temp_dir().join(format!("music-playback-cancel-{}", uuid::Uuid::new_v4()));
        let service = PlaybackService::new(
            context,
            Arc::clone(&catalog),
            Arc::new(registry),
            cache_dir.clone(),
        );
        catalog
            .insert(
                "cancel-test-id".to_string(),
                PlayableEntry {
                    view: TrackView::new(
                        "title".to_string(),
                        "artist".to_string(),
                        String::new(),
                        1.0,
                        "cancel-test-id".to_string(),
                    ),
                    source_ref: SourceRef::Wechat {
                        url: format!("http://{}/audio", address),
                    },
                },
            )
            .await;

        let cancel = CancellationToken::new();
        let task_cancel = cancel.clone();
        let service_clone = Arc::new(service);
        let task = tokio::spawn(async move {
            match service_clone
                .load_track_source("cancel-test-id", &task_cancel)
                .await
            {
                Ok(TrackSource::Progressive { reader, .. }) => {
                    // 模拟解码器：读 spool，取消后应返回错误而不是成功 EOF。
                    let mut reader = reader;
                    let mut buf = Vec::new();
                    std::io::Read::read_to_end(&mut reader, &mut buf).map(|_| buf)
                }
                Ok(TrackSource::File(_)) => Ok(Vec::new()),
                Err(error) => Err(std::io::Error::other(error.to_string())),
            }
        });
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        cancel.cancel();
        assert!(task.await.unwrap().is_err());
        let cache_path = cache_dir.join(format!("{:x}.audio", md5::compute("cancel-test-id")));
        assert!(!tokio::fs::try_exists(cache_path).await.unwrap());
        // writer 清理：临时 spool 文件不残留。
        let mut leftovers = tokio::fs::read_dir(&cache_dir).await.unwrap();
        assert!(leftovers.next_entry().await.unwrap().is_none());
        let _ = tokio::fs::remove_dir_all(cache_dir).await;
        server.abort();
    }
}
