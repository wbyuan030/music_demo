# Implementation Plan: 最近播放

**Branch**: `001-recent-tracks` | **Date**: 2026-06-30 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/001-recent-tracks/spec.md`

---

## Summary

"最近播放"功能的后端数据模型和存储逻辑已完整实现（`RecentTrack` 实体、`add_recent_track`、`list_recent_track`），播放时自动记录也已接入（`music_handler/utils.rs:77`）。

本次工作是**修复前端断裂点 + 补齐后端事件推送**，使端到端流程贯通：
- 播放 → 记录 → 事件推送 → 前端刷新 → 点击重播

---

## Technical Context

参见 `docs/architecture.md`：
- §1 技术栈（Tauri v2, React 19, Zustand 5, native_db 0.8）
- §3.2 播放链路（handle_event → MusicHandler → rodio）
- §3.3 状态架构（Rust OnceLock + Zustand stores 双轨）
- §4.1 数据模型（RecentTrack id=3, version=1）
- §5 关键约束（MAX_RECENT_TRACK_COUNT=100, OnceLock 初始化顺序）
- §6.1 Tauri command 注册模式

本次不引入新依赖。

---

## Constitution Check

*GATE: 无 constitution.md，跳过。若后续建立则以项目现有约定（docs/architecture.md §6）为准。*

---

## Changes

### 后端：1 处新增

| 文件 | 变更 | 对应需求 |
|---|---|---|
| `src-tauri/src/music_handler/utils.rs` | `add_recent_track()` 成功后通过 `app_handle.emit("db_tracks_changed", "recent")` 推送事件 | FR-007 |

### 前端：3 处修复 + 1 处可选

| 文件 | 变更 | 对应需求 |
|---|---|---|
| `src/store/Db.ts` | `useRecentStore.getRecentTracks` 修正 invoke 命令名为 `list_recent_tracks` | FR-005 |
| `src/features/TrackLists.tsx` | onClick 接入 `usePlayerStore.setCurrentTrack` | FR-006 |
| `src/App.tsx` | 恢复路由（`StateEnum` → 页面切换），使 MainPage / TrackPage / SearchPage 可用 | 支撑 US1-US3 |
| `src/components/MainPageContent.tsx` | _(可选)_ 优化空状态文案，保持现有即可 | FR-008 |

---

## Project Structure

无需新建目录。所有变更在现有文件中：

```text
src-tauri/src/music_handler/utils.rs   # 后端事件推送 (1 处 insert)
src/store/Db.ts                        # 修正 command 名 (1 行 fix)
src/features/TrackLists.tsx            # 接入播放 (5 行)
src/App.tsx                            # 恢复路由 (~15 行)
```

---

## Complexity Tracking

> 无违规项。本次是已有代码的修复和补全，不引入新模块或新模式。
