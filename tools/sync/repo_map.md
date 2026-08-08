# repo_map（自动生成 by tools/sync/repo_map.py，勿手改）

> 这是 music_demo 代码的静态地图。翻译上游改动前先读本节，
> 确认我方对应文件的真实形态（常量名/端点/行号），不要凭上游命名猜我方结构。

## 来源清单（sources.json）

- `audius`：prefix=`au`, rust_name=`Audius`, has_search=True, status=adopted

## extractor 代码地图

### audius/
- `audius/mod.rs`: 2 行
- `audius/player.rs`: 126 行
  - 常量: PLAYER_ENDPOINT
  - 端点: https://discoveryprovider.audius.co/v1/tracks
- `audius/search.rs`: 168 行
  - 常量: SEARCH_ENDPOINT
  - 端点: https://api.audius.co/v1/tracks/search
### bilibili/
- `bilibili/mod.rs`: 88 行
  - 端点: https://www.bilibili.com/video/BV1GJ411x7, https://www.bilibili.com/audio/au1003142
- `bilibili/player.rs`: 130 行
  - 端点: https://api.bilibili.com/x/player/playurl?bvid={}&cid={}&fnver=0&fnval=4048&fourk=1, https://www.bilibili.com/
- `bilibili/search.rs`: 150 行
  - 端点: https://api.bilibili.com/x/web-interface/search/type?{}, https://api.bilibili.com/x/web-interface/view?bvid={}
- `bilibili/types.rs`: 174 行
- `bilibili/utils.rs`: 202 行
  - 常量: MIXIN_KEY_ENC_TAB, WBI_KEYS, WBI_TTL_SECS, COOKIE_DONE
  - 端点: https://api.bilibili.com/x/web-interface/nav, https://www.bilibili.com/
### youtube/
- `youtube/api.rs`: 367 行
  - 常量: INNERTUBE_API_KEY, VISITOR_CACHE
  - 端点: https://www.youtube.com/, https://music.youtube.com/youtubei/v1/search?key={}&prettyPrint=false, https://music.youtube.com, https://www.youtube.com/youtubei/v1/player?key={}&prettyPrint=false, https://www.youtube.com, https://www.youtube.com/watch?v={}
- `youtube/commands.rs`: 17 行
- `youtube/mod.rs`: 85 行
  - 端点: https://youtu.be/, https://www.youtube.com/watch?v=dQw4w9WgXcQ, https://youtu.be/dQw4w9WgXcQ, https://music.youtube.com/watch?v=dQw4w9WgXcQ&list=RDAMVMdQw4w9WgXcQ, https://youtu.be/dQw4w9WgXcQ?t=30, https://example.com
- `youtube/player.rs`: 166 行
  - 端点: https://www.youtube.com, https://www.youtube.com/watch?v={}
- `youtube/search.rs`: 240 行
- `youtube/types.rs`: 423 行

## playback adapter 地图

- `playback/audius.rs`: 255 行
- `playback/bilibili.rs`: 53 行
- `playback/wechat.rs`: 63 行
- `playback/youtube.rs`: 53 行

## 上游↔我方映射（youtube client 配置）

- 我方 `WEB_REMIX`（api.rs:96）client_version=`1.20260707.12.00`
- 我方 `ANDROID_VR`（api.rs:154）client_version=`1.65.10`

上游对应：`yt_dlp/extractor/youtube/_base.py` 的 `INNERTUBE_CLIENTS` 键（web_music→WEB_REMIX 等）。
