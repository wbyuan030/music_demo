# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 49
- T1 纯数据: 1 | T2 逻辑: 3 | T3 新来源: 1 | T4 无关: 44

## 需人工确认

- cf68b8f4 [ie/youtube:tab] Always extract channel metadata (#17386): youtube tab 提取器元数据逻辑（我方 youtube/ 模块无对应 _tab extractor，需确认是否有频道/播放列表标签页解析需求）
- 5d5b634d [ie/youtube] Add `web_embedded` client fallbacks (#17462): youtube/api.rs client 配置列表、fallback 逻辑
- dae52d83 [ie/youtube] Remove `android_vr` from default clients (#17461): youtube/api.rs 中的 ANDROID_VR 客户端配置及默认客户端选择逻辑
- 81ecd58b [ie/niconico:channel] Support channels (#17398): 无（music_demo 未集成 niconico 来源）

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
