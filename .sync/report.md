# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 49
- T1 纯数据: 1 | T2 逻辑: 2 | T3 新来源: 0 | T4 无关: 46

## 需人工确认

- bbc809a1 [utils] `devalue`: Improve binary type parsing (#16934): youtube/types.rs 或 youtube/api.rs 中解析 YouTube innerTube 响应的 JS 类型（TypedArray/ArrayBuffer/DataView/BigInt）可能受影响；上游修复了 BigInt64Array/BigUint64Array 的 struct typecode（'l'/'L'→'q'/'Q'）、新增 Float16Array、支持 ArrayBuffer 引用（索引而非内联 base64）、支持 byte offset/length、DataView 正确返回 bytes。若我方 YouTube 响应解析涉及 devalue 反序列化，需参照此逻辑适配。
- 3a08beaf Release 2026.08.19: youtube/api.rs（client 配置）、youtube/search.rs（视频元数据提取）

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
