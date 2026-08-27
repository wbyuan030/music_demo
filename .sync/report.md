# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 41
- T1 纯数据: 1 | T2 逻辑: 2 | T3 新来源: 1 | T4 无关: 37

## 需人工确认

- 5d5b634d [ie/youtube] Add `web_embedded` client fallbacks (#17462): youtube/api.rs 中 WEB_REMIX 等 INNERTUBE_CLIENTS 配置及播放失败时的 client fallback 逻辑；可能影响 age-gated/受限视频在 android_vr/visionos 不可用时的降级路径
- dae52d83 [ie/youtube] Remove `android_vr` from default clients (#17461): youtube/api.rs — WEB_REMIX 和 ANDROID_VR client 配置，以及默认 client 选择逻辑
- 81ecd58b [ie/niconico:channel] Support channels (#17398): 上游新增 niconico channel 提取器，我方代码地图中无任何 niconico 相关 extractor（仅 audius、bilibili、youtube），故对我方无影响。

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
