# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 49
- T1 纯数据: 2 | T2 逻辑: 2 | T3 新来源: 1 | T4 无关: 44

## 需人工确认

- bbc809a1 [utils] `devalue`: Improve binary type parsing (#16934): youtube/api.rs（YouTube innertube API 响应中的 JS 值反序列化）
- dae52d83 [ie/youtube] Remove `android_vr` from default clients (#17461): 我方 youtube/api.rs 中 ANDROID_VR 客户端配置（client_version=1.65.10）及 playback/youtube.rs 中客户端选择逻辑；上游已注明自 2026.08.17 起所有格式均被 403，且从 _DEFAULT_CLIENTS 和 _DEFAULT_JSLESS_CLIENTS 中移除
- 81ecd58b [ie/niconico:channel] Support channels (#17398): 上游新增 niconico channel extractor（ch.nicovideo.jp 视频频道/搜索），music_demo 当前不支持 Niconico 站点；与本方已采用的 Audius、Bilibili、YouTube 无交集

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
