# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 32
- T1 纯数据: 1 | T2 逻辑: 1 | T3 新来源: 1 | T4 无关: 29

## 需人工确认

- dae52d83 [ie/youtube] Remove `android_vr` from default clients (#17461): youtube/api.rs — ANDROID_VR client (`client_version='1.65.10'`) upstream now reports ALL formats 403'd since 2026.08.17; music_demo may need to drop or deprioritize this client.
- 62185352 [ie/tiktok] Support share URLs (#17459): 未实施 - music_demo 不支持 TikTok

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
