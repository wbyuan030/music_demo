# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 75
- T1 纯数据: 3 | T2 逻辑: 5 | T3 新来源: 1 | T4 无关: 66

## 需人工确认

- a51ef56c [ie/youtube] Improve error messages (#17884): youtube/player.rs 或 youtube/api.rs - YouTube 视频不可播放错误消息解析逻辑
- 12ac9aca [ie/youtube] Fix initial data extraction (#17703): youtube/api.rs - _search_json / ytInitialData 提取逻辑；可能影响 visitor token 获取与页面解析路径
- 89734124 [ie/youtube] Set `tv_samsung` as a default client (#17805): youtube/api.rs — DEFAULT_CLIENTS常量、年龄限制视频fallback逻辑（移除tv_downgraded回退）
- 3a08beaf Release 2026.08.19: youtube/: visionos client fallback, channel_follower_count, live adaptive fragments generation; hls: empty fragment list crash; Request Handler: curl_cffi 0.16.x support
- cf68b8f4 [ie/youtube:tab] Always extract channel metadata (#17386): 我方 youtube/_tab.py 的 tab 解析逻辑：channel metadata 提取路径，影响 YoutubeTabIE 中 selected_tab 和 metadata 的赋值行为
- 81ecd58b [ie/niconico:channel] Support channels (#17398): 上游新增 niconico 频道 extractor（ch.nicovideo.jp），与我方 youtube/bilibili 无关

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
