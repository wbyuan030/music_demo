# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 50
- T1 纯数据: 1 | T2 逻辑: 3 | T3 新来源: 0 | T4 无关: 46

## 需人工确认

- 3a08beaf Release 2026.08.19: youtube/api.rs（player client 版本常量）及 youtube/ 下部分解析逻辑
- cf68b8f4 [ie/youtube:tab] Always extract channel metadata (#17386): youtube/mod.rs 或 youtube/player.rs 中的 channel/tab 解析逻辑
- dae52d83 [ie/youtube] Remove `android_vr` from default clients (#17461): youtube/api.rs — ANDROID_VR 客户端仍定义（clientVersion=1.65.10），但上游注释确认该版本自 2026.08.17 起所有格式（含 live HLS 和 itag 18）均返回 403。上游已从默认客户端移除 android_vr，music_demo 需在流选择逻辑中降低其优先级或弃用

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
