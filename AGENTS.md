# AGENTS — 模块索引

## 架构总览

```
Tauri App (src-tauri)
├── lib.rs                 # 入口, Tauri Builder, command 注册
├── extractor/             # yt-dlp 对齐的 Extractor Runtime (核心)
│   ├── protocol.rs        #   Extractor trait, ExtractorResult, ExtractError
│   ├── model.rs           #   RawMediaInfo / Track / PlaybackManifest
│   ├── context.rs         #   ExtractorContext (共享 HTTP client, 取消)
│   ├── youtube/           #   YouTube / YouTube Music 提取器
│   └── bilibili/          #   Bilibili 提取器
├── music_fetch/           # 旧版抓取模块（待迁移）
├── music_handler/         # rodio 音频播放
├── storage.rs             # native_db CRUD
├── types.rs               # TrackView / TrackMeta 等
├── public.rs              # list/toggle Tauri commands
└── global.rs              # OnceLock 全局状态

cli/                       # CLI 测试工具
└── src/main.rs            # search / manifest / download / extract
```

---

## Extractor 协议

### `Extractor` trait (`extractor/protocol.rs`)

```rust
#[async_trait]
pub trait Extractor: Send + Sync {
    fn key(&self) -> &'static str;          // 唯一标识, 如 "youtube:music"
    fn priority(&self) -> i32;              // 匹配优先级, 高值优先
    fn matches(&self, input: &ExtractInput) -> bool;
    async fn extract(&self, input: ExtractInput, context: &ExtractorContext)
        -> Result<ExtractorResult, ExtractError>;
}
```

### 结果类型

| 类型 | 对应 yt-dlp `_type` | 说明 |
|------|---------------------|------|
| `Media(RawMediaInfo)` | `video`（默认） | 单个音视频条目 |
| `Playlist(PlaylistInfo)` | `playlist` | 多个条目的集合 |
| `Redirect(RedirectInfo)` | `url` | 重定向到其他 extractor |
| `TransparentRedirect(..)` | `url_transparent` | 带元数据覆盖的重定向 |
| `MultiMedia(MultiMediaInfo)` | `multi_video` | 多段组成一个节目 |

### 数据模型

**Extractor 层** — yt-dlp 兼容（保留所有原始字段在 `extra` map）：
- `RawMediaInfo` — id, title, formats, entries, extra
- `RawFormat` — url, ext, format_id, acodec, vcodec, tbr, filesize, http_headers...

**应用层** — 稳定的前端模型：
- `Track` — id(`yt:xxx`/`bili:xxx`), title, artists[], album, duration_ms, artwork[]
- `PlaybackManifest` — streams[](url, mime_type, bitrate), headers, expires_at

**转换**：`track_to_raw_media()` / `raw_media_to_track()` 在 provider 模块内实现。

---

## ExtractorContext

所有 Extractor 共享单一 `reqwest::Client`，通过 `ExtractorContext` 传递：

```rust
pub struct ExtractorContext {
    pub http: Arc<Client>,              // 共享 HTTP client (cookie_store)
    pub options: ExtractorOptions,       // UA, 代理, 区域设置
    pub cancellation: CancellationToken, // 取消信号
    pub logger: Arc<dyn ExtractLogger>,  // 日志
}
```

### Provider 特有的 Auth 处理

| Provider | 认证方式 | 缓存策略 |
|----------|---------|----------|
| **YouTube** | `ANDROID_VR` client + `X-Goog-Visitor-Id`（首页提取） | Visitor data 15min 缓存 |
| **Bilibili** | WBI 签名（img_key + sub_key 从 nav API 获取） | WBI key 30s 缓存 |

---

## Provider 实现

### YouTube (`extractor/youtube/`)

| 文件 | 职责 |
|------|------|
| `api.rs` | InnerTube API 客户端（search / player）+ visitor data 管理 |
| `search.rs` | YouTube Music 搜索 → `Vec<Track>` |
| `player.rs` | ANDROID_VR + visitor data 获取音频流 → `PlaybackManifest` |
| `commands.rs` | `search_youtube_music()`, `get_youtube_manifest()` Tauri commands |
| `types.rs` | InnerTube 请求/响应类型（约 30 个 struct） |
| `mod.rs` | `YouTubeMusicExtractor` impl + `extract_video_id()` |

**API 端点**：
- Search: `POST music.youtube.com/youtubei/v1/search` (WEB_REMIX client)
- Player: `POST www.youtube.com/youtubei/v1/player` (ANDROID_VR client)
- Fallback: 页面 scraping `ytInitialPlayerResponse`

### Bilibili (`extractor/bilibili/`)

| 文件 | 职责 |
|------|------|
| `utils.rs` | WBI 签名, cookie 管理, 共享 headers |
| `search.rs` | Bilibili 视频搜索 → `Vec<Track>` |
| `player.rs` | DASH playurl API → `PlaybackManifest` |
| `types.rs` | Bilibili API 响应类型 |
| `mod.rs` | `BiliBiliExtractor` impl + `extract_bili_video_id()` + Tauri commands |

**API 端点**：
- Search: `GET api.bilibili.com/x/web-interface/search/type` (WBI 签名)
- Video Info: `GET api.bilibili.com/x/web-interface/view`
- Play URL: `GET api.bilibili.com/x/player/playurl`
- Audio: `GET www.bilibili.com/audio/music-service-c/web/url`

---

## CLI 工具 (`cli/`)

独立 binary，依赖 `app_lib` crate。

```
cargo run -p music-cli -- <command>

Commands:
  search <query>             搜索（--source youtube|bilibili|all）
  manifest <video-id>        YouTube 音频 manifest
  info <video-id>            YouTube 视频信息
  manifest-bili <bvid>       Bilibili 音频 manifest
  download <video-id>        下载 YouTube 音频到本地
  extract <url>              通用 extractor 调试

Options:
  --source <SOURCE>          搜索源 [default: all] [youtube, bilibili]
  --section <SECTION>        YouTube 板块 [default: songs]
  --format <FORMAT>          输出格式 [default: table] [table, json]
  -o, --output <PATH>        下载输出路径
```

---

## 添加新的 Provider

1. 在 `extractor/` 下创建目录（如 `extractor/netease/`）
2. 实现 `Extractor` trait（`mod.rs`）
3. 实现 `search`/`player` 模块（使用 `ExtractorContext.http`）
4. 添加 `commands.rs`（`#[tauri::command]` 函数）
5. 在 `lib.rs` 注册 command
6. （可选）在 CLI 添加对应子命令

```rust
// extractor/netease/mod.rs
pub struct NeteaseExtractor;

#[async_trait]
impl Extractor for NeteaseExtractor {
    fn key(&self) -> &'static str { "netease" }
    fn matches(&self, input: &ExtractInput) -> bool {
        input.url.starts_with("netease:")
    }
    async fn extract(&self, input: ExtractInput, ctx: &ExtractorContext)
        -> Result<ExtractorResult, ExtractError> { ... }
}
```
