# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 50
- T1 纯数据: 1 | T2 逻辑: 2 | T3 新来源: 0 | T4 无关: 47

## 需人工确认

- 5d5b634d [ie/youtube] Add `web_embedded` client fallbacks (#17462): youtube/api.rs — client fallback 策略：_DEFAULT_AUTHED_CLIENTS 新增 web_embedded；_DEFAULT_PREMIUM_CLIENTS 顺序调整；unplayable 时追加 web_embedded 作为 tv_downgraded 前的备选客户端
- dae52d83 [ie/youtube] Remove `android_vr` from default clients (#17461): youtube/api.rs - ANDROID_VR 客户端配置及默认客户端选择逻辑

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
