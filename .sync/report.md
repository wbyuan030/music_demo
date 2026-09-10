# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 48
- T1 纯数据: 2 | T2 逻辑: 1 | T3 新来源: 0 | T4 无关: 45

## 需人工确认

- bbc809a1 [utils] `devalue`: Improve binary type parsing (#16934): youtube/api.rs 中的 innertube API 响应解析；上游改进了 ArrayBuffer/TypedArray/DataView 的字节偏移、元素对齐、Float16Array 等二进制类型解析逻辑

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
