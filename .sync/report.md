# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 61
- T1 纯数据: 1 | T2 逻辑: 3 | T3 新来源: 0 | T4 无关: 57

## 需人工确认

- c2901ed3 [rh:urllib] Fix relative redirects with dot segments (#17712): 网络层重定向处理逻辑，可能影响 YouTube/Bilibili extractor 的 HTTP 请求层（若 music_demo 有自建重定向处理或 URL 规范化逻辑）
- c7fb478d [ie/youtube] Use Safari UA for web_embedded client (#17684): youtube/api.rs 客户端配置 + youtube/search.rs 或 player.rs 中的格式优先级逻辑
- dae52d83 [ie/youtube] Remove `android_vr` from default clients (#17461): youtube/api.rs 中的 ANDROID_VR 客户端配置及默认客户端选择逻辑

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
