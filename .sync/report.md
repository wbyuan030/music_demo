# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 49
- T1 纯数据: 1 | T2 逻辑: 2 | T3 新来源: 0 | T4 无关: 46

## 需人工确认

- bbc809a1 [utils] `devalue`: Improve binary type parsing (#16934): youtube/api.rs 或 youtube/types.rs 中 JavaScript 序列化数据的反序列化逻辑
- 3a08beaf Release 2026.08.19: youtube/api.rs (WEB_REMIX/ANDROID_VR client 配置、版本更新)、youtube/search.rs/player.rs (解析逻辑变更)

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
