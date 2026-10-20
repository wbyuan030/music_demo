# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 63
- T1 纯数据: 4 | T2 逻辑: 2 | T3 新来源: 0 | T4 无关: 57

## 需人工确认

- c7fb478d [ie/youtube] Use Safari UA for web_embedded client (#17684): youtube/api.rs — web_embedded client config + format source_preference logic
- bbc809a1 [utils] `devalue`: Improve binary type parsing (#16934): youtube/api.rs 中的 devalue 解析逻辑（JS 序列化数据反序列化）。新增 Float16Array、DataView、ArrayBuffer 引用解析，修复 BigInt64Array/BigUint64Array 格式码，ArrayBuffer 改为返回 bytes。这些改动影响 YouTube player 响应的 binary data 解析路径。

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
