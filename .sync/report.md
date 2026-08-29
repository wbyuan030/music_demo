# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 42
- T1 纯数据: 3 | T2 逻辑: 1 | T3 新来源: 0 | T4 无关: 38

## 需人工确认

- cf68b8f4 [ie/youtube:tab] Always extract channel metadata (#17386): youtube/player.rs / youtube/search.rs — _extract_metadata_from_tabs 逻辑，影响从 tab 响应中提取频道元数据的字段解析路径

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
