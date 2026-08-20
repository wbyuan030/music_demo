# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 29
- T1 纯数据: 1 | T2 逻辑: 2 | T3 新来源: 0 | T4 无关: 26

## 需人工确认

- 3a08beaf Release 2026.08.19: youtube/api.rs 中的 player client 配置（WEB_REMIX、ANDROID_VR 等常量及 client_version）
- 5d5b634d [ie/youtube] Add `web_embedded` client fallbacks (#17462): youtube/api.rs 中 INNERTUBE_CLIENTS 配置和默认客户端选择逻辑

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
