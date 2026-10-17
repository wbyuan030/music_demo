# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 63
- T1 纯数据: 3 | T2 逻辑: 3 | T3 新来源: 0 | T4 无关: 57

## 需人工确认

- bbc809a1 [utils] `devalue`: Improve binary type parsing (#16934): youtube/api.rs - devalue 解析改进，涉及 ArrayBuffer/TypedArray/DataView 的二进制解析逻辑
- 5d5b634d [ie/youtube] Add `web_embedded` client fallbacks (#17462): youtube/api.rs — 客户端 fallback 逻辑
- dae52d83 [ie/youtube] Remove `android_vr` from default clients (#17461): youtube/api.rs — ANDROID_VR client configuration (clientVersion=`1.65.10`)及默认客户端列表;android_vr 客户端自 2026.08.17 起全部格式返回 403

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
