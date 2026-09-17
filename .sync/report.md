# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 50
- T1 纯数据: 0 | T2 逻辑: 2 | T3 新来源: 0 | T4 无关: 48

## 需人工确认

- cf68b8f4 [ie/youtube:tab] Always extract channel metadata (#17386): youtube/api.rs 或 youtube/search.rs 中的元数据提取逻辑，涉及 channel metadata 的 `_extract_metadata_from_tabs` 调用时机
- dae52d83 [ie/youtube] Remove `android_vr` from default clients (#17461): youtube/api.rs — INNERTUBE_CLIENTS 配置；android_vr 的 PLAYER_PO_TOKEN_POLICY 由 required=False 升级为

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
