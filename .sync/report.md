# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 49
- T1 纯数据: 2 | T2 逻辑: 1 | T3 新来源: 0 | T4 无关: 46

## 需人工确认

- dae52d83 [ie/youtube] Remove `android_vr` from default clients (#17461): youtube/api.rs — 影响我方 DEFAULT_CLIENTS / default client 选择逻辑

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
