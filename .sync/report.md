# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 49
- T1 纯数据: 3 | T2 逻辑: 1 | T3 新来源: 0 | T4 无关: 45

## 需人工确认

- cf68b8f4 [ie/youtube:tab] Always extract channel metadata (#17386): youtube/mod.rs 或 youtube/types.rs - YouTube tab/频道元数据提取逻辑

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
