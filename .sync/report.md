# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 49
- T1 纯数据: 0 | T2 逻辑: 2 | T3 新来源: 0 | T4 无关: 47

## 需人工确认

- cf68b8f4 [ie/youtube:tab] Always extract channel metadata (#17386): youtube/tab 提取逻辑 - 需确认我方是否有频道/播放列表多 tab 处理
- dae52d83 [ie/youtube] Remove `android_vr` from default clients (#17461): youtube/api.rs - INNERTUBE_CLIENTS 中 ANDROID_VR 客户端的状态及默认客户端选择逻辑

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
