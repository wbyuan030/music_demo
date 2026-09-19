# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 50
- T1 纯数据: 2 | T2 逻辑: 3 | T3 新来源: 0 | T4 无关: 45

## 需人工确认

- c7fb478d [ie/youtube] Use Safari UA for web_embedded client (#17684): youtube/api.rs - INNERTUBE_CLIENTS 配置及 HLS 流选择逻辑
- bbc809a1 [utils] `devalue`: Improve binary type parsing (#16934): youtube/types.rs 中的 JS devalue 解析逻辑（如 ArrayBuffer/TypedArray 解码），上游修复了 Float16Array 支持、byte offset/length 对齐、ArrayBuffer 引用等，若我方依赖 devalue 解析 YouTube player 响应中的二进制数据字段，需要同步适配
- dae52d83 [ie/youtube] Remove `android_vr` from default clients (#17461): 我方 youtube/api.rs 中 WEB_REMIX/ANDROID_VR client 配置及默认客户端选择逻辑。上游移除了 android_vr 从 _DEFAULT_CLIENTS/_DEFAULT_JSLESS_CLIENTS，并更新了注释说明自 2026.08.17 起 android_vr client (v1.65.10) 对所有格式（含 live HLS 和 itag 18）返回 403。需评估是否同步移除 android_vr 默认行为或保留为 fallback。

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
