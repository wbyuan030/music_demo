# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 47
- T1 纯数据: 0 | T2 逻辑: 3 | T3 新来源: 1 | T4 无关: 43

## 需人工确认

- fcdbefb8 [utils] `subs_list_to_dict`: Fix empty value handling (#17311): yt_dlp/utils/traversal.py → music_demo 字幕解析路径/字段处理逻辑（如 audius/search.rs 或 bilibili/player.rs 中调用 subs_list_to_dict 的片段）
- cf68b8f4 [ie/youtube:tab] Always extract channel metadata (#17386): youtube/api.rs, youtube/player.rs — channel tab parsing logic
- dae52d83 [ie/youtube] Remove `android_vr` from default clients (#17461): youtube/api.rs 中的 ANDROID_VR 客户端配置需删除或标记失效；默认客户端选择逻辑受影响
- 81ecd58b [ie/niconico:channel] Support channels (#17398): 新增 niconico channel extractor，对应 yt_dlp/extractor/niconico.py，支持 ch.nicovideo.jp 站点

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
