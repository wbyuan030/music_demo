# Implementation Plan: 音频质量量化测试体系

**Branch**: `002-audio-backend` | **Date**: 2026-07-01 | **Design**: [design.md](./design.md)

---

## Summary

新建 `src-tauri/src/audio_quality/` 模块，包含三个子模块（signal / metrics / probe），不修改现有 `music_handler` 公开 API，不引入新 Cargo 依赖。

---

## Technical Context

参见 `docs/architecture.md`：
- §3.2 播放链路（spawn_progress 集成点）
- §5 已知问题（WSL2 音频颗粒感）

---

## Changes

### 新建文件

| 文件 | 内容 | 行数估计 |
|---|---|---|
| `src-tauri/src/audio_quality/mod.rs` | 模块声明 + test 入口 | 15 |
| `src-tauri/src/audio_quality/signal.rs` | 参考信号生成 (WAV header + 正弦波) | 50 |
| `src-tauri/src/audio_quality/metrics.rs` | 静音段/振幅跳变/谐波信噪比 | 100 |
| `src-tauri/src/audio_quality/probe.rs` | 在线播放探针 | 80 |

### 修改文件

| 文件 | 变更 | 行数 |
|---|---|---|
| `src-tauri/src/lib.rs` | 添加 `mod audio_quality;` | 1 |
| `src-tauri/src/music_handler/handler.rs` | spawn_progress 中注入 probe.tick() | ~5 |
| `src-tauri/src/music_handler/utils.rs` | 增强 diagnose_cached_audio_quality 测试 | ~30 |

---

## Project Structure

```
src-tauri/src/
├── audio_quality/       ← 新建
│   ├── mod.rs
│   ├── signal.rs
│   ├── metrics.rs
│   └── probe.rs
├── music_handler/
│   ├── handler.rs       ← +probe 集成
│   └── utils.rs         ← +测试增强
└── lib.rs               ← +mod audio_quality
```

---

## Complexity Tracking

无违规项。不引入新依赖，不改变现有 API。
