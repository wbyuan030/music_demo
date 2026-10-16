# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 63
- T1 纯数据: 6 | T2 逻辑: 1 | T3 新来源: 1 | T4 无关: 55

## 需人工确认

- cf68b8f4 [ie/youtube:tab] Always extract channel metadata (#17386): youtube/api.rs - channel metadata extraction logic in tab/playlist resolution
- 81ecd58b [ie/niconico:channel] Support channels (#17398): 上游新增 niconico channel 支持，但我方 music_demo 代码地图中无 niconico extractor 实现（仅 audius/bilibili/youtube），当前不影响我方。

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
