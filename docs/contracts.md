# Contracts: command / event / 状态同步 / 持久化

> 稳定契约文档。所有名称以代码为准，改动任何一项都属于破坏性变更。
> 架构总览见 [architecture.md](./architecture.md)；新增来源见 [extension-guide.md](./extension-guide.md)。

---

## 1. 前端 commands

全部注册在 `lib.rs` 的 `invoke_handler`，实现位于 `public.rs`（`parse_track_from_wx` 在 `music_fetch/wx.rs`，统一 URL 解析在 `music_fetch/url.rs`，调试 manifest 命令在 extractor 目录）。

| Command | Payload | 返回 | 用途 |
|---|---|---|---|
| `search_music` | `{ keyword: string, source?: string }` | `TrackView[]` | 统一多来源搜索。`source` 取值 `youtube` / `bilibili` / `audius`，缺省或 `"all"` 搜索全部已注册来源；未知来源名报错；空 keyword 返回空数组 |
| `handle_event` | `{ event: string }`（JSON 字符串） | `()` | 播放控制，action 见第 2 节 |
| `list_recent_tracks` | 无 | `TrackView[]` | 最近播放（最多 100 条） |
| `list_liked_tracks` | 无 | `TrackView[]` | 收藏列表 |
| `toggle_liked_track` | `{ id: string }` | `()` | 切换收藏；曲目不在 DB 时先从 catalog upsert 曲目再切换 |
| `get_playback_state` | 无 | `PlaybackStateView` | 播放状态快照，挂载/重连对账用（见第 4 节） |
| `report_frontend_log` | `{ level, source, message, stack?, command? }` | `()` | 前端日志转发，后端以 target `frontend` 落日志（见 observability.md） |
| `parse_track_from_wx` | `{ url: string }` | `TrackView` | 解析微信公众号文章内嵌音乐，entry 写入 catalog |
| `parse_track_from_url` | `{ url: string }` | `TrackView` | 统一解析 WeChat / YouTube / Bilibili URL；Audius URL 当前返回明确不支持错误；entry 写入 catalog |
| `get_youtube_manifest` | `{ video_id: string }` | `PlaybackManifest` | **调试入口**，不是 UI 播放契约 |
| `get_bilibili_manifest` | `{ bvid: string }` | `PlaybackManifest` | **调试入口**，不是 UI 播放契约 |
| `get_cache_info` | 无 | `CacheInfo` | 缓存目录中已完成 `.audio` 文件的数量和字节数；不统计 spool 临时文件 |
| `clear_cache` | 无 | `CacheInfo` | 清理可移除缓存；当前播放文件及其兼容路径受保护，不改变播放状态 |
| `list_playlists` | 无 | `PlaylistView[]` | 列出用户播放列表及其中曲目 |
| `create_playlist` | `{ name: string }` | `PlaylistView` | 创建播放列表；空名称报错 |
| `rename_playlist` | `{ id: string, name: string }` | `PlaylistView` | 重命名播放列表 |
| `delete_playlist` | `{ id: string }` | `()` | 删除播放列表关系，不删除曲目、收藏或最近播放记录 |
| `add_playlist_track` | `{ playlistId: string, trackId: string }` | `PlaylistView` | 向播放列表添加已知曲目；重复添加保持幂等 |
| `remove_playlist_track` | `{ playlistId: string, trackId: string }` | `PlaylistView` | 从播放列表移除曲目 |
| `reorder_playlist_track` | `{ playlistId: string, trackId: string, position: number }` | `PlaylistView` | 事务性调整播放列表曲目顺序，位置从 0 开始 |

`PlaybackStateView`（`music_handler/status.rs`，camelCase 序列化）：

```ts
{
  phase: "idle" | "loading" | "playing" | "paused";
  trackId: string | null;
  positionSecs: number;
  error: string | null;
}
```

UI 播放只发送 `handle_event({ action: "play", id })`，不直接调用调试 manifest 命令。

队列元数据由前端 `QueueStore` 保存到 `localStorage["music_demo.playback_queue"]`，只保存曲目、循环模式和随机模式；应用重启后恢复队列但不自动播放，也不恢复 `currentIndex`。
全局键盘交互由 `App` 挂载的 `useKeyboardShortcuts` 负责，不新增 command 或 event：

| 按键 | 行为 |
|---|---|
| `Space` / `MediaPlayPause` | 播放 / 暂停 |
| `←` / `→` | 前后跳转 5 秒，结果限制在曲目时长范围内 |
| `↑` / `↓` | 音量 ±5，范围为 `0..50` |
| `M` | 静音 / 取消静音 |
| `N` / `MediaTrackNext` | 下一首 |
| `P` / `MediaTrackPrevious` | 上一首 |
| `L` | 收藏 / 取消收藏 |

快捷键在 `input`、`textarea`、`select`、`contenteditable` 内以及 Ctrl/Meta/Alt 组合键下不生效；只有实际处理的按键会阻止浏览器默认行为，避免影响表单输入和文本编辑。

前端复合类型：

```ts
type CacheInfo = { fileCount: number; bytes: number }
type PlaylistView = {
  id: string
  name: string
  trackCount: number
  tracks: TrackView[]
}
```


---

## 2. handle_event actions

`event` 是 JSON 字符串，解析后按 `action` 分发（`music_handler/publics.rs` → `MusicState`）。缺 `action` 或 action 未知均报错；`play` 缺 `id`、`seek` 缺 `time`、`volume` 缺 `volume` 也报错。

| action | payload | 后端行为 |
|---|---|---|
| `play` | `{ action: "play", id: string }` | 取消旧任务、清空 sink、置 loading，启动加载流程 |
| `pause` | `{ action: "pause" }` | `sink.pause()`，状态置 paused |
| `recovery` | `{ action: "recovery" }` | `sink.play()`，状态置 playing（前端恢复播放用） |
| `volume` | `{ action: "volume", volume: number }` | 实际音量 = `volume / 50` |
| `seek` | `{ action: "seek", time: number }`（秒） | 见下方排队语义 |
| `quit` | `{ action: "quit" }` | 取消任务、`sink.stop()`，状态置 idle |

### seek 排队语义

- sink 为空（曲目下载/加载中）时，seek 写入 `pending_seek`，后到的 seek 覆盖先到的；
- 解码器 append 后、`play_start` 之前立即 `try_seek` 应用；
- 新的 `play` 请求清空 `pending_seek`；
- 锁顺序固定：`pending_seek` 先于 sink。

---

## 3. 后端 events

由 `MusicHandler` 经 `app_handle.emit` 发射。事件名称与 payload 是外部契约；新增诊断信息优先写日志，不改这些 payload。

| Event | Payload | 触发时机 |
|---|---|---|
| `play_start` | 无 | decoder 打开 + Sink.append + 排队 seek 应用之后 |
| `play_failed` | `string`（错误消息） | 加载、解码、decoder task 失败；**取消不触发**，被新请求取代不算失败 |
| `play_progress` | `{ secs, nanos }`（std `Duration` 序列化，前端读 `secs`） | sink 非空时每 500ms |
| `play_end` | 无 | sink 自然结束 |
| `play_probe_report` | JSON 字符串 | 自然结束且 `stall_count > 0` 时发射（见 observability.md） |
| `db_tracks_changed` | `"recent"` | 播放成功持久化到 recent 列表后 |

---

## 4. 播放状态同步模型（SSOT）

后端是播放状态的**唯一状态源**。`PlaybackStatus`（`music_handler/status.rs`）持有 phase / track_id / last_error，所有变更都发生在事件发射点，保证快照与事件流一致。

```text
命令改状态    handle_event(play / pause / recovery / seek / quit)
事件推状态    play_start / play_failed / play_end / play_progress
快照对账      get_playback_state：挂载、重连、监听器注册晚于事件时兜底
超时兜底      前端 loading 超过 30s 未收到终态事件 → 判定失败
```

规则：

- 前端单曲状态是「事件流 + 快照」的投影（`store/Player.ts`）；播放顺序由 `store/Queue.ts` 管理，队列只通过 `handle_event(play)` 驱动后端单曲播放；
- 乐观更新只允许用于延迟掩盖（如拖动条 currentTime）；播放控制命令失败时不得提交本地状态，状态最终由后端事件/快照纠正；
- 加载失败必须发 `play_failed`（取消不发），否则前端会永久卡在 loading；
- 前端超时兜底：`LOAD_TIMEOUT_MS = 30_000`，触发 `onPlaybackFailed("加载超时")`。

---

## 5. 数据持久化与兼容约束

### 5.1 native model（`storage.rs`）

| Model | native id / version | 字段 | 作用 |
|---|---|---|---|
| `TrackDbItem` | 1 / 1 | `title`, `artist`, `cover_url`, `duration: f32`, `id`(PK), `src`(本地缓存路径，可为空), `meta: TrackMeta` | 曲目元数据 + 稳定 source reference + 缓存路径 |
| `LikedTrack` | 2 / 1 | `id`(PK), `added_at`(secondary, i64) | 收藏关系 |
| `RecentTrack` | 3 / 1 | `id`(PK), `added_at`(secondary, i64) | 最近播放，最多 100 条（`MAX_RECENT_TRACK_COUNT`） |
| `Playlist` | 4 / 1 | `id`(PK), `name`, `created_at`(secondary, i64) | 用户播放列表 |
| `PlaylistTrack` | 5 / 1 | `id`(PK), `playlist_id`(secondary), `track_id`, `position`(secondary, i64) | 播放列表与曲目的有序关系 |

缓存文件路径：`<temp>/music_cache/<md5(稳定 TrackId)>.audio`，命中前必须通过 Decoder 打开校验。`get_cache_info` 只统计常规文件且扩展名为 `.audio` 的完成缓存；`clear_cache` 不删除当前播放曲目的稳定路径和 DB 中的兼容缓存路径。

### 5.2 兼容约束

- native model 的 `id/version` 一旦发布不可修改；
- 旧数据兼容：`MetaValue::Wechat(String)` 记录经 `to_source_ref()` 还原为 `SourceRef::Wechat` 直接播放；`MetaValue::Bili(_)` 仅保留为**序列化墓碑**（旧记录可读、列表可显示，但不再可播，播放报 TrackNotFound）；`MetaValue::Extractor("yt:…")` / `("bili:…")` / `("srm:…")` 对应来源前缀（见 extension-guide.md 前缀表）；
- 新来源只保存稳定 reference 与本地缓存路径，不保存短期 manifest URL（`src` 字段不写网络 URL）；
- DB 初始化失败时直接失败退出，不能删除或重建旧库；
- 缓存文件先经 Decoder 校验再视为有效命中，损坏文件自动走重新下载。

### 5.3 测试隔离

- 应用入口 `init_db()` 等价于 `init_db_at("./local.db")`（redb 文件锁，单进程独占）；
- 测试用 `init_db_at(<临时路径>)` 指向独立临时 DB，不占用应用的 `local.db`；`storage.rs` 单测在独立路径建库并事后清理。
