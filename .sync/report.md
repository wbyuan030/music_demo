# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 49
- T1 纯数据: 2 | T2 逻辑: 2 | T3 新来源: 0 | T4 无关: 45

## 需人工确认

- bbc809a1 [utils] `devalue`: Improve binary type parsing (#16934): youtube/api.rs 或 youtube/types.rs 中的JavaScript内嵌数据解析逻辑
- cf68b8f4 [ie/youtube:tab] Always extract channel metadata (#17386): youtube/api.rs / youtube/search.rs 中的 tab/playlist 元数据提取逻辑

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
