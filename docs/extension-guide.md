# Extension Guide: 新增音乐来源

> 完整指南：adapter 模型、注册步骤、测试要求、"不要修改什么"。
> 前提：先读 [architecture.md](./architecture.md) 的播放链路与 [contracts.md](./contracts.md) 的契约。

---

## 1. 来源 adapter 模型

一个来源对应**一个 adapter 类型**，同时实现两个 trait：

- `SearchProvider`（`playback/search.rs`）：搜索能力，返回 `Vec<PlayableEntry>`；
- `PlaybackResolver`（`playback/resolver.rs`）：解析能力，从 `PlayableEntry` 返回 `PlaybackManifest`。

同一 adapter 类型会同时注册进 `ResolverRegistry` 和 `SearchRegistry`。只有解析能力、没有搜索入口的来源（现有 `WechatResolver`）只实现 `PlaybackResolver`。

adapter 只做三件事：

1. 调用来源自己的 search / player extractor（`extractor/<source>/`，共享 `ExtractorContext`）；
2. 把来源结果转成统一模型：搜索 → `PlayableEntry`，播放 → `PlaybackManifest`；
3. 保持稳定边界：来源专属字段停留在 extractor model 或 `SourceRef` 变体中，在 resolver 边界转换掉，不让它泄漏进 `PlaybackManifest` 或前端。

### 完整代码骨架

```rust
// src-tauri/src/playback/xxx.rs
use async_trait::async_trait;

use crate::extractor::{context::ExtractorContext, model::PlaybackManifest, xxx};

use super::{
    model::{PlayableEntry, SourceKind, SourceRef},
    resolver::{PlaybackError, PlaybackResolver},
    search::{track_to_entry, SearchProvider},
};

/// Xxx 来源适配器：搜索 + 播放解析共用同一套来源抽象。
pub struct XxxSource;

#[async_trait]
impl SearchProvider for XxxSource {
    fn source(&self) -> SourceKind {
        SourceKind::Xxx
    }

    async fn search(
        &self,
        keyword: &str,
        context: &ExtractorContext,
    ) -> Result<Vec<PlayableEntry>, PlaybackError> {
        let tracks = xxx::search::search_music(context, keyword, None).await?;
        Ok(tracks
            .into_iter()
            .filter_map(|track| track_to_entry(track, SourceKind::Xxx))
            .collect())
    }
}

#[async_trait]
impl PlaybackResolver for XxxSource {
    fn source(&self) -> SourceKind {
        SourceKind::Xxx
    }

    async fn resolve(
        &self,
        entry: &PlayableEntry,
        context: &ExtractorContext,
    ) -> Result<PlaybackManifest, PlaybackError> {
        let SourceRef::Xxx { id } = &entry.source_ref else {
            return Err(PlaybackError::NoResolver(
                SourceKind::Xxx.as_str().to_string(),
            ));
        };
        xxx::player::get_manifest(context, id).await.map_err(Into::into)
    }
}
```

---

## 2. 稳定标识扩展（model.rs / types.rs）

新增来源需要扩展以下标识层，全部集中在 `playback/model.rs` 与 `types.rs`：

1. `SourceKind` 增加枚举臂 + `as_str()`；
2. `TrackId` 的 `Display` / `FromStr` 增加前缀解析（见第 4 节前缀表）；
3. `SourceRef` 增加新变体；
4. `TrackMeta::from_source_ref` / `to_source_ref` 增加对应转换（新来源存 `MetaValue::Extractor("<prefix>:<id>")`，读取时还原成 `SourceRef`）。

---

## 3. 注册步骤

只在 `playback/runtime.rs` 注册，其余核心模块零改动：

```rust
let mut registry = ResolverRegistry::new();
registry.register(YoutubeSource);
registry.register(BilibiliSource);
registry.register(WechatResolver);
registry.register(XxxSource); // 新增：解析能力
let resolvers = Arc::new(registry);

let mut search = SearchRegistry::new();
search.register(YoutubeSource);
search.register(BilibiliSource);
search.register(XxxSource); // 新增：搜索能力（可选）
let search = Arc::new(search);
```

注意：

- 只解析、无搜索的来源只注册到 `ResolverRegistry`；
- `SearchRegistry` 的注册顺序 = `search_music` 的 `"all"` 枚举顺序；
- `search_music` 按注册表枚举来源，新来源自动被 `"all"` 包含——**不要**新增独立的前端搜索 command，也不要改 `public.rs` 的分发。

---

## 4. track_to_entry 前缀表

`playback/search.rs` 的 `track_to_entry` 把 extractor 的 `Track` 转成 `PlayableEntry`，按 `source` 去掉 ID 前缀。**新增来源必须在这里加 arm**，否则搜索永远过滤掉该来源。

| SourceKind | Track ID 前缀 | `TrackId` FromStr | 备注 |
|---|---|---|---|
| Youtube | `yt:` | `yt:<remote_id>` | |
| Bilibili | `bili:` | `bili:<remote_id>` | |
| Wechat | 无 | 无 | 搜索侧返回 `None` 跳过；播放走 `SourceRef::Wechat { url }` |

另外两点：

- 搜索返回的 `PlayableEntry` 必须带稳定 source reference（如 `SourceRef::Youtube { video_id }`），供 `search_music` 合并去重和后续播放；
- extractor 返回的 `Track.id` 必须带来源前缀（如 `format!("yt:{}", video_id)`），否则被 `track_to_entry` 过滤。

---

## 5. 测试要求

每个新来源至少要有：

1. **search contract tests**：`search` 返回的 `PlayableEntry` ID 带正确前缀、`source_ref` 匹配来源；
2. **resolver tests**：`resolve` 返回 `streams` 非空的 `PlaybackManifest`（空流/空 URL 必须走错误路径）；
3. **manifest 测试**：由 extractor 层覆盖（流选择、headers、过期时间）；
4. **command contract tests**：command 名称稳定（`public.rs` 已有先例：`__tauri_command_name_*!` 断言，如 `search_music`）；
5. 涉及 DB 的测试用 `init_db_at` 指向独立临时 DB，不占用应用的 `local.db`；
6. 至少一次真实启动/调用 smoke（播放一条新来源曲目）。

---

## 6. 不要修改什么

新增来源**禁止**修改：

- `PlaybackService` 的下载和缓存逻辑（`playback/service.rs`）；
- `MusicHandler` 的 Sink 生命周期（`music_handler/handler.rs`）；
- 前端的播放 URL 处理与播放状态机（`store/Player.ts`、`components/MiniPlayer.tsx`）；
- `PlaybackManifest` 加来源专属字段——来源专属信息放在来源自己的 extractor model 或 `SourceRef` 中，在 resolver 边界转换掉；
- `public.rs` 的 `search_music` 分发逻辑（注册表已自动包含新来源）。

如果来源需要专属字段，放在来源自己的 extractor model 或 `SourceRef` 变体中，在 resolver 边界转换掉。
