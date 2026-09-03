# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 50
- T1 纯数据: 1 | T2 逻辑: 3 | T3 新来源: 0 | T4 无关: 46

## 需人工确认

- 3a08beaf Release 2026.08.19: 影响我方 youtube/player.rs 中的客户端版本配置和端点逻辑
- 5d5b634d [ie/youtube] Add `web_embedded` client fallbacks (#17462): youtube/api.rs - default client selection logic for authed/premium accounts; player fallback chain for age-gated/unplayable videos
- dae52d83 [ie/youtube] Remove `android_vr` from default clients (#17461): youtube/api.rs - ANDROID_VR client may need to be removed from default client list; also update the comment noting 403 enforcement since 2026.08.17

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
