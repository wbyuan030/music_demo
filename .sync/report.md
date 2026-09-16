# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 50
- T1 纯数据: 4 | T2 逻辑: 1 | T3 新来源: 0 | T4 无关: 45

## 需人工确认

- bbc809a1 [utils] `devalue`: Improve binary type parsing (#16934): 可能影响 youtube/api.rs 中 devalue 格式的解析；该 commit 改进了 ArrayBuffer 引用、TypedArray（Float16Array/DataView）的字节偏移与长度处理逻辑

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
