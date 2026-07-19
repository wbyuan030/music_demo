# Cross-Platform Media Extractor Spec

## 1. Goal

为 Tauri 音乐播放器实现一个可运行于 Desktop、Android 和 iOS 的媒体搜索与解析内核。

系统不直接依赖 yt-dlp binary，而是：

- 对齐 yt-dlp 的 Extractor 协议与结果模型
- 使用 Rust 实现跨端 Extractor Runtime
- 将 yt-dlp 作为开发环境与 CI 中的参考实现和测试 Oracle
- 支持通过 Workflow 持续跟踪和迁移 yt-dlp 上游变化

首期仅支持 YouTube / YouTube Music 的音乐搜索、元数据获取和音频播放地址解析。

---

## 2. Scope

### In Scope

- 关键词音乐搜索
- 搜索结果分页
- 单曲元数据解析
- Playlist 与 Redirect 结果
- 音频 Format 提取与筛选
- HTTP Headers、Cookies 和 Session 管理
- Android Media3 与 iOS AVPlayer 播放
- 与 yt-dlp 结果进行差分测试

### Out of Scope

- 完整迁移 yt-dlp 全部站点
- 视频下载与本地保存
- FFmpeg 转码和音视频合并
- 完整复制 yt-dlp CLI
- 自动翻译任意 Python Extractor 为 Rust
- 绕过 DRM

---

## 3. Architecture

```text
Tauri UI
   │
   ▼
Application Media Model
Track / Album / Playlist / PlaybackManifest
   │
   ▼
Normalization Layer
   │
   ▼
Extractor Runtime
   ├── Registry
   ├── Resolver
   ├── HTTP Context
   ├── Cookie Store
   ├── Cache
   └── Script Runtime
   │
   ▼
Provider Extractors
   └── YouTube / YouTube Music
```

yt-dlp 仅运行于开发环境和 CI：

```text
yt-dlp
  │
  ▼
Oracle JSON
  │
  ▼
Differential Test
  │
  ├── Metadata Comparison
  ├── Format Comparison
  ├── Request Validation
  └── Migration Report
```

---

## 4. Extractor Protocol

```rust
#[async_trait]
pub trait Extractor: Send + Sync {
    fn key(&self) -> &'static str;

    fn priority(&self) -> i32 {
        0
    }

    fn matches(&self, input: &ExtractInput) -> bool;

    async fn initialize(
        &self,
        context: &ExtractorContext,
    ) -> Result<(), ExtractError> {
        Ok(())
    }

    async fn extract(
        &self,
        input: ExtractInput,
        context: &ExtractorContext,
    ) -> Result<ExtractorResult, ExtractError>;
}
```

结果模型：

```rust
pub enum ExtractorResult {
    Media(MediaInfo),
    Playlist(PlaylistInfo),
    Redirect(RedirectInfo),
    TransparentRedirect(TransparentRedirectInfo),
    MultiMedia(MultiMediaInfo),
}
```

必须支持递归 Redirect 解析，并限制最大跳转深度。

---

## 5. Data Model

Extractor 层保留 yt-dlp 兼容字段：

```rust
pub struct RawMediaInfo {
    pub id: Option<String>,
    pub title: Option<String>,
    pub formats: Vec<RawFormat>,
    pub entries: Vec<RawMediaInfo>,
    pub extra: serde_json::Map<String, serde_json::Value>,
}
```

应用层使用稳定的业务模型：

```rust
pub struct Track {
    pub id: String,
    pub title: String,
    pub artists: Vec<String>,
    pub album: Option<String>,
    pub duration_ms: Option<u64>,
    pub artwork: Vec<Image>,
}

pub struct PlaybackManifest {
    pub streams: Vec<AudioStream>,
    pub headers: HeaderMap,
    pub expires_at: Option<SystemTime>,
}
```

UI 不得直接依赖 yt-dlp 兼容字段。

---

## 6. Runtime Context

所有 Extractor 必须使用统一 Context，不得自行创建 HTTP Client。

```rust
pub struct ExtractorContext {
    pub http: Arc<dyn HttpClient>,
    pub cookies: Arc<dyn CookieStore>,
    pub cache: Arc<dyn CacheStore>,
    pub scripts: Arc<dyn ScriptRuntime>,
    pub logger: Arc<dyn ExtractLogger>,
    pub cancellation: CancellationToken,
    pub options: ExtractorOptions,
}
```

Runtime 应统一处理：

- User-Agent
- Referer
- Cookies
- Proxy
- Retry
- Rate Limit
- Cache
- Region 与 Language
- Request Logging
- Cancellation

---

## 7. yt-dlp Alignment Rules

必须对齐：

- Extractor 匹配与选择机制
- Extractor 生命周期
- Media、Playlist、Redirect 等结果类型
- Format、Headers 和 Metadata 语义
- Error 与 Warning 分类
- Oracle JSON 输出

不要求对齐：

- Python 类继承结构
- yt-dlp CLI 参数系统
- Downloader 与 Postprocessor
- Python 动态类型和内部工具函数实现

原则：

> Compatible boundaries, Rust-native internals.

---

## 8. Migration Workflow

CI 定期跟踪 yt-dlp 指定 Extractor 的变化。

Workflow 应执行：

1. 拉取指定 yt-dlp 版本
2. 对固定 Query 和 URL 运行 yt-dlp
3. 生成标准化 Oracle JSON
4. 运行 Rust Extractor
5. 比较结果
6. 验证音频 URL 是否可访问
7. 输出差异报告

差异至少分为：

- URL 匹配变化
- API 参数变化
- JSON Path 变化
- Metadata 映射变化
- Format 变化
- Signature 或 Script 变化
- Authentication 变化

首期只生成报告，不自动合并代码。

---

## 9. MVP

MVP 完成标准：

- 支持 YouTube Music 关键词搜索
- 支持搜索结果分页
- 支持解析单曲标题、作者、时长和封面
- 支持生成可播放的音频 Manifest
- Desktop、Android 和 iOS 使用同一 Rust Extractor
- Android 使用 Media3 播放
- iOS 使用 AVPlayer 播放
- 至少维护 20 个固定测试 Query 和 URL
- 核心字段与 yt-dlp Oracle 差分测试通过
- 音频 URL 可成功请求部分字节

---

## 10. Suggested Repository Structure

```text
crates/
├── media-model/
├── extractor-protocol/
├── extractor-runtime/
├── extractor-youtube/
├── extractor-ytdlp-compat/
└── tauri-plugin-player/

tests/
├── fixtures/
├── oracle/
└── differential/

.github/
└── workflows/
    └── extractor-sync.yml
```

---

## 11. Success Criteria

项目成功不以支持站点数量衡量，而以以下指标衡量：

- Extractor 是否可独立维护
- yt-dlp 上游变化是否能被快速发现
- 差异是否能定位到具体解析阶段
- Desktop 和 Mobile 是否共享相同核心逻辑
- UI 是否与 Provider 实现解耦
- 新增 Provider 是否无需修改 Runtime 核心
