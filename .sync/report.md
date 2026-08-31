# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 49
- T1 纯数据: 2 | T2 逻辑: 3 | T3 新来源: 2 | T4 无关: 42

## 需人工确认

- bbc809a1 [utils] `devalue`: Improve binary type parsing (#16934): 我方 youtube/api.rs 中的 devalue 解析逻辑（JavaScript TypedArray/ArrayBuffer 反序列化），影响 YouTube 播放器响应中二进制数据（如 signature token、cipher 参数等）的解析
- fcdbefb8 [utils] `subs_list_to_dict`: Fix empty value handling (#17311): 影响我方字幕解析逻辑：yt_dlp 修复了 subs_list_to_dict 中 id/ext 为空字符串时的处理。music_demo 自有 extractor 可能需要同步类似的空值防御逻辑
- 3a08beaf Release 2026.08.19: youtube/api.rs（client 配置常量）、youtube/player.rs（live adaptive fragments 逻辑）、youtube/types.rs（channel_follower_count 字段）
- 81ecd58b [ie/niconico:channel] Support channels (#17398): 上游新增 niconico channel 支持（NiconicoChannelIE），我方代码库无 niconico extractor，暂时无直接冲突或移植需求
- 62185352 [ie/tiktok] Support share URLs (#17459): 上游新增 TikTok share URL 支持，我方 music_demo 无 TikTok extractor（仅 audius/bilibili/youtube），需评估是否新增 src-tauri/src/tiktok/ 模块并适配 URL 匹配逻辑

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
