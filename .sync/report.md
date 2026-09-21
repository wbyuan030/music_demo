# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 50
- T1 纯数据: 2 | T2 逻辑: 2 | T3 新来源: 0 | T4 无关: 46

## 需人工确认

- c7fb478d [ie/youtube] Use Safari UA for web_embedded client (#17684): youtube/api.rs（client 配置）+ youtube/player.rs（格式偏好逻辑）
- 3a08beaf Release 2026.08.19: youtube/api.rs — client config + live adaptive fragment logic + visionos/web_embedded fallbacks

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
