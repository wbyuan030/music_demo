# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 61
- T1 纯数据: 3 | T2 逻辑: 3 | T3 新来源: 0 | T4 无关: 55

## 需人工确认

- c7fb478d [ie/youtube] Use Safari UA for web_embedded client (#17684): youtube/api.rs 或 player.rs（INNERTUBE_CLIENTS 配置 + web_embedded client 的 HLS 格式降级逻辑）
- bbc809a1 [utils] `devalue`: Improve binary type parsing (#16934): youtube/api.rs - JavaScript response parsing (devalue utility for ArrayBuffer/TypedArray parsing)
- cf68b8f4 [ie/youtube:tab] Always extract channel metadata (#17386): youtube/search.rs 或 youtube/player.rs 中解析频道/playlist tab 的元数据提取逻辑

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
