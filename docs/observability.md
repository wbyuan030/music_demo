# Observability: 播放 trace、欠载探针、前端日志

> 播放链路问题的诊断手册。架构背景见 [architecture.md](./architecture.md)，事件契约见 [contracts.md](./contracts.md)。

---

## 1. 播放 trace（`playback/trace.rs`）

每次播放请求生成一个 `trace_id`（uuid v4），`PlaybackTrace` 关联 `track_id` 与起始时间，所有阶段日志使用同一组字段。日志 target 为 `playback_trace`。

行格式：

```text
trace_id=<id> track_id=<id> stage=<stage> elapsed_ms=<ms> <details...>
```

核心字段：

| 字段 | 含义 |
|---|---|
| `trace_id` | 一次播放请求的关联 ID |
| `track_id` | 曲目 ID（含来源前缀，如 `yt:xxx`） |
| `stage` | 当前阶段（见下） |
| `elapsed_ms` | 距请求开始的时间 |
| details | `status=...` 及阶段专属 key-value（错误消息、字节数、时长等） |

### stage 列表

| stage | 含义 |
|---|---|
| `request` / `previous_task` | 收到播放请求 / 取消旧任务 |
| `load` | `PlaybackService` 加载（start / ok / error / cancelled） |
| `db_lookup` / `entry` | DB 查询与 entry 定位 |
| `cache` | 缓存检查：`location=db\|stable`，`status=hit\|miss\|invalid`，带字节数 |
| `resolver` | 来源解析（含过期刷新：`expired_refresh` / `refresh_ok`） |
| `download` / `http` / `http_body` | 候选流枚举、HTTP 响应、body 写入 |
| `cache_commit` | 下载完成后的原子提交（rename）或中止 |
| `stream` | spool 流式路径启动 |
| `decode` | Decoder 打开（ok / error / cancelled，记录 duration、channels、sample_rate） |
| `sink` | Sink.append 结果 |
| `play_start` / `play_end` | 事件发射点 |
| `progress` | 每 5s 一次的进度指标（位置 + 欠载统计） |
| `underrun` | 严重欠载（gap > 500ms） |
| `persist` | recent / 缓存元数据持久化 |
| `control` | seek / pause / recovery / volume / quit |
| `search` | `search_music` 单来源失败 |

### 诊断顺序

```text
没有 resolver       → source / entry / command 问题
resolver 成功无 HTTP → 流选择或取消问题
HTTP 成功无 decode   → cache / container / codec 问题
decode 成功无 sink   → handler / task 问题
sink + progress 正常但无声 → 系统输出设备 / 音量 / CoreAudio
progress 有大 gap    → 欠载或调度问题
```

不要记录带签名参数的音频 URL；只记录 MIME、bitrate、content length 和 URL hash。

---

## 2. 欠载探针（`audio_quality/probe.rs` + `instrumented_sink.rs`）

探针以**装饰器**形式接入：`InstrumentedSink` 包装 `rodio::Sink`，业务代码（`music_handler/handler.rs`）只感知播放接口与两个生命周期钩子，不接触探针细节：

```text
begin_track(track_id, trace_id)    play 请求时开启观测
get_pos() → Duration               内部完成 tick + 暂停感知 + 5s 指标日志
take_report() → Option<ProbeReport> 自然结束时取汇总、清探针、输出 play_end 日志
```

`PlaybackProbe` 测量逻辑：

- `tick(current_pos, paused)`：**暂停时位置冻结是正常行为，不构成欠载**——只重置基线（`last_pos` / 墙钟），避免恢复播放后误报；
- gap = 期望位置 − 实际位置，`> 100ms` 记为一次 stall，`> 500ms` 记为 severe（同时打 `stage=underrun` 的 warn 日志）；
- 每 5s 输出一次 `progress` 指标（`should_emit_metric`）；
- 自然结束时装饰器输出 `play_end` 汇总日志：`stall_count / severe_count / max_gap_ms / avg_gap_ms`；若 `stall_count > 0`，handler 额外向前端发射 `play_probe_report`。

`play_probe_report` 的 payload 是 JSON 字符串（时长字段单位均为**秒**）：

```json
{ "stall_count": 3, "severe_count": 1, "max_gap": 0.812, "avg_gap": 0.26, "stall_timeline": [[12.5, 0.812]] }
```

`stall_timeline` 为 `[elapsed_sec, gap_sec]` 数组。

---

## 3. 前端日志转发

### 3.1 转发路径（`src/services/frontendLog.ts`）

- 包装 console 方法（error / warn / info / debug / log），保留原始 console 输出；
- 捕获 `window.error` 与 `unhandledrejection`；
- 字段截断到 4000 字符（`MAX_FIELD_LENGTH`），结构化后直接 `invoke("report_frontend_log", …)`。

### 3.2 关键约束

- 日志上报**不能用 `safeInvoke`**：上报失败会再次进入错误上报，造成递归。`forwardFrontendLog` 直连 `invoke` 并吞掉失败；
- `safeInvoke`（`src/services/invoke.ts`）失败时：错误写入 Toast store（`store/Error.ts`）并调用 `forwardFrontendLog`（`source="safeInvoke"`，携带 `command` 字段）；
- 后端 `report_frontend_log` 按 level 映射到 `log` crate，target 为 `frontend`。

### 3.3 日志文件位置

- `tauri-plugin-log` 只在 debug 构建启用（`log::LevelFilter::Info`）；
- macOS 开发日志：

```text
~/Library/Logs/com.ferrisMusic/music_demo.log
```

- release 若需要持久化前端日志，应单独设计日志保留策略，不要在 command 中直接写临时文件。
