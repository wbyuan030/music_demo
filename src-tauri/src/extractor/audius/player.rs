use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use crate::extractor::context::ExtractorContext;
use crate::extractor::model::{AudioStream, PlaybackManifest};
use crate::extractor::protocol::ExtractError;
use serde::Deserialize;

/// Audius 播放解析端点（默认生产 URL；测试用 options.endpoints 覆盖）。
/// 实际请求路径为 {endpoint}/{id}。
pub const PLAYER_ENDPOINT: &str = "https://discoveryprovider.audius.co/v1/tracks";

#[derive(Debug, Deserialize)]
struct TrackResponse {
    data: TrackData,
}

#[derive(Debug, Deserialize)]
struct TrackData {
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    title: String,
    #[serde(default)]
    stream: Option<StreamInfo>,
}

#[derive(Debug, Deserialize)]
struct StreamInfo {
    url: String,
}

/// Audius 播放解析：track id → PlaybackManifest（stream.url 为签名直链）。
pub async fn get_manifest(
    ctx: &ExtractorContext,
    id: &str,
) -> Result<PlaybackManifest, ExtractError> {
    let base = ctx.options.endpoint("player", PLAYER_ENDPOINT);
    let url = format!("{}/{}?app_name=music_demo", base.trim_end_matches('/'), id);
    let resp: TrackResponse = ctx
        .http
        .get(&url)
        .send()
        .await
        .map_err(|e| ExtractError::NetworkError(e.to_string()))?
        .json()
        .await
        .map_err(|e| ExtractError::ParseError(format!("audius player: {}", e)))?;

    let stream_url = resp.data.stream.map(|s| s.url).ok_or_else(|| {
        ExtractError::ExtractionFailed(format!("audius track {} has no stream", id))
    })?;

    let mut headers = HashMap::new();
    headers.insert("User-Agent".to_string(), ctx.options.user_agent.clone());

    Ok(PlaybackManifest {
        streams: vec![AudioStream {
            url: stream_url,
            mime_type: "audio/mpeg".to_string(),
            bitrate: None,
            codec: Some("mp3".to_string()),
            content_length: None,
        }],
        headers,
        expires_at: Some(SystemTime::now() + Duration::from_secs(3600)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractor::context::ExtractorOptions;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    /// 行为测试 2：来源能解析出可播放流（mock，CI 门禁）。
    #[tokio::test]
    async fn can_resolve_manifest_via_mock() {
        let raw = include_str!("fixtures/player.json");
        assert_ne!(
            raw.trim(),
            "{}",
            "fixtures/player.json 未录制：先跑 record_fixture.py"
        );

        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .unwrap();
        let addr = listener.local_addr().unwrap();
        let body = raw.to_string();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 4096];
            let _ = socket.read(&mut buf).await;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(), body
            );
            socket.write_all(resp.as_bytes()).await.unwrap();
        });

        let ctx =
            ExtractorContext::with_client(reqwest::Client::builder().no_proxy().build().unwrap())
                .with_options(
                    ExtractorOptions::default()
                        .with_endpoint("player", &format!("http://{}", addr)),
                );
        let manifest = get_manifest(&ctx, "jaKgV").await.unwrap();
        assert!(!manifest.streams.is_empty(), "无流");
        assert!(
            manifest.streams[0].url.starts_with("http"),
            "流 URL 非法: {}",
            manifest.streams[0].url
        );
        let _ = server;
    }

    /// 真实流量 smoke：真实解析一个已知 ID（夜间/发布前 job 跑）。
    #[tokio::test]
    #[ignore = "真实网络，由 nightly smoke job 运行"]
    async fn real_site_player_smoke() {
        let ctx = ExtractorContext::with_client(reqwest::Client::new());
        let manifest = get_manifest(&ctx, "jaKgV").await.unwrap();
        assert!(!manifest.streams.is_empty(), "真实播放解析返回空流");
    }
}
