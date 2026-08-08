use crate::extractor::context::ExtractorContext;
use crate::extractor::model::{Image, Track};
use crate::extractor::protocol::ExtractError;
use serde::Deserialize;

/// Audius 搜索端点（默认生产 URL；测试用 options.endpoints 覆盖）。
pub const SEARCH_ENDPOINT: &str = "https://api.audius.co/v1/tracks/search";

#[derive(Debug, Deserialize)]
struct SearchResponse {
    data: Vec<TrackItem>,
}

#[derive(Debug, Deserialize)]
struct TrackItem {
    id: String,
    title: String,
    duration: Option<u64>,
    #[serde(default)]
    artwork: Option<Artwork>,
    #[serde(default)]
    user: Option<User>,
}

#[derive(Debug, Deserialize)]
struct Artwork {
    #[serde(rename = "150x150")]
    small: Option<String>,
    #[serde(rename = "480x480")]
    medium: Option<String>,
    #[serde(rename = "1000x1000")]
    large: Option<String>,
}

#[derive(Debug, Deserialize)]
struct User {
    name: Option<String>,
}

/// Audius 搜索：keyword → Track 列表。
pub async fn search_music(
    ctx: &ExtractorContext,
    keyword: &str,
    limit: Option<u32>,
) -> Result<Vec<Track>, ExtractError> {
    let url = ctx.options.endpoint("search", SEARCH_ENDPOINT);
    let resp: SearchResponse = ctx
        .http
        .get(&url)
        .query(&[
            ("query", keyword),
            ("limit", &limit.unwrap_or(10).to_string()),
        ])
        .send()
        .await
        .map_err(|e| ExtractError::NetworkError(e.to_string()))?
        .json()
        .await
        .map_err(|e| ExtractError::ParseError(format!("audius search: {}", e)))?;

    Ok(resp
        .data
        .into_iter()
        .map(|item| {
            let artist = item.user.and_then(|u| u.name).unwrap_or_default();
            let artwork = item
                .artwork
                .map(|a| {
                    let mut imgs = Vec::new();
                    if let Some(u) = a.large {
                        imgs.push(Image {
                            url: u,
                            width: Some(1000),
                            height: Some(1000),
                        });
                    }
                    if let Some(u) = a.medium {
                        imgs.push(Image {
                            url: u,
                            width: Some(480),
                            height: Some(480),
                        });
                    }
                    if let Some(u) = a.small {
                        imgs.push(Image {
                            url: u,
                            width: Some(150),
                            height: Some(150),
                        });
                    }
                    imgs
                })
                .unwrap_or_default();
            Track {
                id: format!("au:{}", item.id),
                title: item.title,
                artists: if artist.is_empty() {
                    Vec::new()
                } else {
                    vec![artist]
                },
                album: None,
                duration_ms: item.duration.map(|d| d * 1000),
                artwork,
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractor::context::ExtractorOptions;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    /// 行为测试 1：来源能搜索（mock，CI 门禁）。
    #[tokio::test]
    async fn can_search_via_mock() {
        let raw = include_str!("fixtures/search.json");
        assert_ne!(
            raw.trim(),
            "{}",
            "fixtures/search.json 未录制：先跑 record_fixture.py"
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
                        .with_endpoint("search", &format!("http://{}", addr)),
                );
        let tracks = search_music(&ctx, "test", Some(3)).await.unwrap();
        assert!(!tracks.is_empty(), "搜索返回空");
        assert!(
            tracks[0].id.starts_with("au:"),
            "ID 前缀错误: {}",
            tracks[0].id
        );
        assert!(!tracks[0].title.is_empty(), "title 为空");
        assert!(tracks[0].artwork.len() >= 1, "artwork 缺失");
        let _ = server;
    }

    /// 真实流量 smoke：验证真实站点当前可用（夜间/发布前 job 跑）。
    #[tokio::test]
    #[ignore = "真实网络，由 nightly smoke job 运行"]
    async fn real_site_search_smoke() {
        let ctx = ExtractorContext::with_client(reqwest::Client::new());
        let tracks = search_music(&ctx, "test", Some(3)).await.unwrap();
        assert!(!tracks.is_empty(), "真实搜索返回空");
    }
}
