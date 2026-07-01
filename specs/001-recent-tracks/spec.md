# Feature Specification: 最近播放

**Feature Branch**: `001-recent-tracks`

**Created**: 2026-06-30

**Status**: Draft

**Input**: 用户希望有一个"最近播放"功能——播放过的歌曲自动出现在首页最近播放列表中，可以点击再次播放，列表按时间倒序。

---

## User Scenarios & Testing

### User Story 1 — 播放后自动记录 (Priority: P1)

用户在任何入口播放一首歌（搜索、URL 解析等），该歌曲自动出现在首页「最近播放」区域。

**Why this priority**: 这是最近播放功能的核心——必须先有记录，才能有展示和交互。没有它，P2/P3 没有数据源。

**Independent Test**: 播放一首歌，然后刷新首页，确认该歌曲出现在最近播放列表中。

**Acceptance Scenarios**:

1. **Given** 最近播放列表为空，**When** 用户通过 URL 解析播放一首微信/B站歌曲，**Then** 该歌曲立即出现在首页「Recent Tracks」列表中
2. **Given** 最近播放列表已有若干歌曲，**When** 用户播放一首已在列表中的旧歌，**Then** 该歌曲被移到列表顶部（时间更新），不会出现重复
3. **Given** 最近播放列表已有 100 首，**When** 用户播放一首新歌，**Then** 最旧的那首被移除，新歌出现在顶部，总数不超过 100

---

### User Story 2 — 点击最近播放再次播放 (Priority: P2)

用户在首页「Recent Tracks」区域点击任意歌曲卡片，可以立即播放该歌曲。

**Why this priority**: 记录是为了重播。用户的核心行为闭环是"看到历史 → 点击 → 播放"。

**Independent Test**: 在最近播放列表中点击一首歌曲，确认播放器开始播放该歌曲。

**Acceptance Scenarios**:

1. **Given** 最近播放列表有至少一首歌曲，**When** 用户点击该歌曲卡片，**Then** MiniPlayer 显示该歌曲信息并开始播放
2. **Given** 用户正在播放歌曲 A，**When** 用户点击最近播放中的歌曲 B，**Then** 停止 A 的播放，切换为播放 B

---

### User Story 3 — 最近播放列表实时刷新 (Priority: P2)

当一首新歌被添加到最近播放时，已经在首页的用户无需手动刷新即可看到列表更新。

**Why this priority**: 用户体验——如果必须切页面再回来才能看到更新，功能是半残的。

**Independent Test**: 保持首页打开，通过搜索播放一首歌，切回首页确认最近播放列表已自动更新。

**Acceptance Scenarios**:

1. **Given** 用户在首页查看最近播放列表，**When** 通过其他入口播放一首新歌并返回首页，**Then** 列表自动更新，新歌出现在顶部
2. **Given** 用户在首页且最近播放为空，**When** 播放第一首歌，**Then** 最近播放区域从空状态变为展示该歌曲

---

### Edge Cases

- 歌曲被从本地缓存清除后，点击最近播放中的该歌曲应如何处理？（当前行为：`parse_track_request` 回退到重新下载——可接受，但需要验证）
- 网络断开时播放的歌曲是否仍然记录？（当前行为：播放发生在下载完成后，所以网络断开不影响已缓存歌曲的记录）
- 最近播放中的歌曲被收藏后又取消收藏，最近播放列表是否受影响？（不应受影响——最近播放和收藏是独立维度）

---

## Requirements

### Functional Requirements

- **FR-001**: 系统 MUST 在每次歌曲成功开始播放时，将该歌曲记录到最近播放列表
- **FR-002**: 系统 MUST 将最近播放列表按播放时间倒序排列（最新播放的排最前）
- **FR-003**: 系统 MUST 限制最近播放列表最多 100 条，超出时自动移除最旧的记录
- **FR-004**: 重复播放已有歌曲时，系统 MUST 更新该记录的时间戳并将其移至顶部，而非创建重复条目
- **FR-005**: 用户 MUST 能够在首页看到最近播放列表（歌曲标题、艺人、封面）
- **FR-006**: 用户 MUST 能够通过点击最近播放列表中的歌曲来重新播放它
- **FR-007**: 系统 MUST 在最近播放列表变更时推送事件通知前端，使 UI 实时更新
- **FR-008**: 最近播放列表为空时，UI MUST 展示有意义的空状态提示

### Key Entities

- **RecentTrack**: 最近播放记录。关键属性：曲目 ID、播放时间戳。关联 `TrackDbItem` 通过相同的 ID。
- **Track**: 曲目本身（标题、艺人、封面 URL、时长）。已在 `docs/architecture.md` §4 中定义。

---

## Success Criteria

### Measurable Outcomes

- **SC-001**: 从播放一首歌到它出现在最近播放列表的端到端延迟 < 2 秒
- **SC-002**: 用户在最近播放列表中点击歌曲到播放开始的延迟 < 1 秒
- **SC-003**: 最近播放列表 UI 更新延迟 < 500ms（事件推送后到 UI 渲染完成）
- **SC-004**: 100% 的播放事件被正确记录（无遗漏）

---

## Assumptions

- 后端 `RecentTrack` 数据模型和 `add_recent_track()` / `list_recent_track()` 函数已实现且正确（参见 `docs/architecture.md` §4.1）
- 前端 `TrackCard` 组件可复用，无需修改
- `MiniPlayer` 组件已支持 `setCurrentTrack` 接口
- 项目已有 Zustand 和 Tauri event 机制，本次不引入新依赖
