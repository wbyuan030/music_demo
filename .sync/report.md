# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 60
- T1 纯数据: 3 | T2 逻辑: 2 | T3 新来源: 1 | T4 无关: 54

## 需人工确认

- bbc809a1 [utils] `devalue`: Improve binary type parsing (#16934): youtube/api.rs 或 youtube/player.rs 中的 JavaScript 数据反序列化逻辑（如 devalue/JSON 解析 YouTube 内嵌二进制数据）
- 5d5b634d [ie/youtube] Add `web_embedded` client fallbacks (#17462): youtube/api.rs（client 配置）、youtube/player.rs（fallback 逻辑）
- d3ea2cf8 [ie/whyp] Fix extractor (#17467): 上游新增 WhypIE extractor（whyp.it 音乐平台），music_demo 当前无此来源

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
