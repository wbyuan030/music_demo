# Tasks: 最近播放

**Input**: `specs/001-recent-tracks/plan.md`, `specs/001-recent-tracks/spec.md`

**Prerequisites**: plan.md (required), spec.md (required)

**Tests**: 手动端到端验证（项目无自动化测试框架）

---

## Phase 1: Setup

无。所有变更在现有文件中，无需新建目录或依赖。

**Checkpoint**: 跳过。

---

## Phase 2: Foundational

**Purpose**: 后端事件推送——前端实时刷新 (US3) 的前提。

- [x] T001 [P] Emit `db_tracks_changed` event after `add_recent_track()` in `src-tauri/src/music_handler/utils.rs`
  - 需要将 `app_handle: tauri::AppHandle` 参数传入 `parse_track_request()`

**Checkpoint**: 播放一首歌后，Rust 侧验证事件已发出（检查日志或前端 console）

---

## Phase 3: User Story 1 — 播放后自动记录 (Priority: P1)

**Goal**: 用户播放歌曲后，首页最近播放列表正确展示。

**Independent Test**: 播放一首歌 → 刷新首页 → 确认歌曲出现在 Recent Tracks 区域。

### Implementation

- [x] T002 [US1] Fix `useRecentStore.getRecentTracks` command name in `src/store/Db.ts`

- [x] T003 [US1] Restore page routing in `src/App.tsx`

**Checkpoint**: 播放一首歌后进入主页，手动刷新页面，歌曲出现在 Recent Tracks 列表中。

---

## Phase 4: User Story 2 — 点击再次播放 (Priority: P2)

**Goal**: 点击最近播放列表中的歌曲可以播放。

**Independent Test**: 在最近播放列表中点击歌曲卡片 → MiniPlayer 显示歌曲信息并开始播放。

### Implementation

- [x] T004 [US2] Wire `onClick` in `src/features/TrackLists.tsx` to `usePlayerStore.setCurrentTrack`

**Checkpoint**: 点击最近播放列表中任意歌曲，MiniPlayer 开始播放该歌曲。

---

## Phase 5: User Story 3 — 实时刷新 (Priority: P2)

**Goal**: 用户播放新歌后，首页列表无需手动刷新即可看到更新。

**Dependency**: T001 (backend emit) 必须先完成。

**Independent Test**: 保持首页打开，播放一首歌，切回首页确认列表已自动更新。

### Implementation

- [x] T005 [US3] Verify `useTrackLists` hook's `db_tracks_changed` listener works end-to-end

**Checkpoint**: 播放一首歌后返回首页，列表自动更新，新歌出现在顶部。

---

## Phase 6: Polish

- [x] T006 Manual end-to-end smoke test

---

## Dependencies & Execution Order

```
T001 (backend emit)
  └─→ T005 (verify real-time refresh)

T002 (fix command name) ─┐
T003 (restore routing)  ─┤─→ US1 可验证
                          │
T004 (wire onClick)      ─┘─→ US2 可验证

T005 (verify refresh)        → US3 可验证

T006 (smoke test)            → 全部验证后
```

**Parallel**: T001, T002, T003, T004 无相互依赖，可并行。
