# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 63
- T1 纯数据: 3 | T2 逻辑: 3 | T3 新来源: 0 | T4 无关: 57

## 需人工确认

- c7fb478d [ie/youtube] Use Safari UA for web_embedded client (#17684): youtube/api.rs 客户端配置 + youtube/search.rs 或 player.rs 格式偏好处理
- cf68b8f4 [ie/youtube:tab] Always extract channel metadata (#17386): music_demo youtube/search.rs 和 youtube/api.rs —— 渠道元数据提取逻辑（self._extract_metadata_from_tabs）确保所有 tab 路径均被调用
- dae52d83 [ie/youtube] Remove `android_vr` from default clients (#17461): 我方 `youtube/api.rs` 中的 `ANDROID_VR` 客户端配置（client_version=`1.65.10`）因上游自 2026.08.17 起对该版本实施全格式 403 限制而失效；同时上游将 `android_vr` 从默认客户端列表中移除，可能影响我方的播放器回退逻辑

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
