# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 49
- T1 纯数据: 1 | T2 逻辑: 3 | T3 新来源: 0 | T4 无关: 45

## 需人工确认

- bbc809a1 [utils] `devalue`: Improve binary type parsing (#16934): youtube/api.rs - INNERTUBE 响应中内嵌的 ArrayBuffer/TypedArray 数据解析（如 po_token 二进制 blob、访客标识等）
- dae52d83 [ie/youtube] Remove `android_vr` from default clients (#17461): youtube/api.rs - ANDROID_VR 客户端从默认列表移除，因 2026.08.17 起所有格式 403
- b375e1d8 [ie/tiktok] Fix extractor (#17452): 核心逻辑：`_generate_blockbuster_headers()` 静态方法——HTTP header 指纹随机化，用于绕过 403 封锁；新增于 common.py，已迁移 dailymotion.py 调用，已在 tiktok.py 两处应用

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
