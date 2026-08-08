# 提案：yt-dlp → music_demo YouTube extractor 内容映射同步（字段层）

> 调研基准：yt-dlp commit `fdcc954df`（2026-07-23）；music_demo 当前工作区。
> 范围：哪些值/逻辑对应、能否自动提取、用什么载体同步、怎么检测漂移、怎么验证。**不改任何代码。**

## 0. 关键调研发现（先读，影响后续所有设计）

1. **API key 已从 yt-dlp 移除，无法从 yt-dlp 同步**。`git log --all -S "AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8"` 证实该 key 最后一次出现在 commit `0e539617a`（`[ie/youtube] Player client maintenance (#10573)`，2024-07-30），该提交删除了硬编码 key。当前 yt-dlp 的 `_call_api`（`_base.py:830-848`）对 `key` query 参数用 `filter_dict(..., cndn=lambda _, v: v)` 过滤——**默认不发 key**，除非 `--extractor-args youtube:innertube_key=XXX`。music_demo `api.rs:7` 硬编码的 `AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8` 与 yt-dlp 现在**没有任何常量对应**。替代同步源：页面 ytcfg 仍在发布 `INNERTUBE_API_KEY`（`downloader/youtube_live_chat.py:155` 读取 `ytcfg['INNERTUBE_API_KEY']` 证实），但那是运行时数据、非仓库常量。
2. **music_demo 的 client 配置与当前 yt-dlp `INNERTUBE_CLIENTS` 逐字一致**：WEB_REMIX `1.20260707.12.00`、ANDROID_VR `1.65.10`/sdk 32/`12L` 与 `_base.py:133-141`、`_base.py:228-240` 完全相同——证实是手工从这份 checkout 抄的。**但 yt-dlp 里这些值只是 fallback 默认**：`_download_ytcfg`（`_base.py:991-1029`）每次从 `https://www.youtube.com` 拉页面 ytcfg 并用其覆盖 clientVersion（`_extract_client_version` 优先取页面值，`_base.py:726-729`）。music_demo 是纯硬编码。
3. **yt-dlp 目录结构**：本 checkout **没有 `_innertube.py`、没有 `_player.py`**。Innertube 逻辑全在 `yt_dlp/extractor/youtube/_base.py`，player/签名/po_token 逻辑全在 `_video.py`。映射表按实际文件写。
4. **yt-dlp 的签名/po_token 是「Python 编排 + 外部 JS 求解器」架构，Rust 无法直接复用**：n/sig 挑战经 `jsc/_director.py` 派发给 Bun/Deno/Node/QuickJS 运行时执行 `jsc/_builtin/vendor/yt.solver.*.js`（`jsc/_builtin/vendor/_info.py` 的 HASHES 校验）；po_token 经 `pot/_director.py` + `pot/_builtin/webpo_cachespec.py`（BotGuard/WebPO HTTP 交互）。这些属于 C 级。
5. **music_demo 现有测试全部离线**（纯函数单测，无网络），网络验证走 `cli/` 子命令（见 §5）。

## 1. 完整映射表（yt-dlp ↔ music_demo）

| # | 项 | yt-dlp 位置（`yt_dlp/extractor/youtube/`） | music_demo 位置（`src-tauri/src/extractor/youtube/`） | 现状差距 |
|---|---|---|---|---|
| 1 | innertube API key | **无**（2024-07 已删；仅 `--extractor-args youtube:innertube_key` 可选，`_base.py:845-846`；运行时源：页面 ytcfg `INNERTUBE_API_KEY`，`downloader/youtube_live_chat.py:155`） | `api.rs:7` `const INNERTUBE_API_KEY: &str = "AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8"`，拼接进 `api.rs:109`、`api.rs:167` 的 URL | 无上游常量可同步；需自行决定保留硬编码 or 运行时提取 |
| 2 | clientName/clientVersion（search=WEB_REMIX） | `_base.py:133-141` `INNERTUBE_CLIENTS['web_music']`（clientName `WEB_REMIX`、clientVersion `1.20260707.12.00`）；同名运行时覆盖：`_download_ytcfg` `_base.py:991-1029` | `api.rs:104-105` `search()` 内联构造 `ClientContext { client_name: "WEB_REMIX", client_version: "1.20260707.12.00", .. }` | 值一致；yt-dlp 会运行时刷新版本号，music_demo 不会 |
| 3 | clientName/clientVersion（player=ANDROID_VR） | `_base.py:228-240` `INNERTUBE_CLIENTS['android_vr']`（`ANDROID_VR`/`1.65.10`/`androidSdkVersion: 32`/`osName: Android`/`osVersion: 12L`/userAgent） | `api.rs:162-169` `player()` 内联构造 `ClientContext { client_name: "ANDROID_VR", client_version: "1.65.10", android_sdk_version: Some(32), os_name: Some("Android"), os_version: Some("12L") }` | 值一致；缺 userAgent（yt-dlp 会发完整 VR UA，music_demo 不发） |
| 4 | `INNERTUBE_CONTEXT_CLIENT_NAME`（数值，用于 `X-YouTube-Client-Name` header） | `_base.py:141`（web_music=67）、`_base.py:241`（android_vr=28）、`_base.py:105`（WEB=1）…；header 组装 `generate_api_headers` `_base.py:964-965` | **无对应**——music_demo 不发 `X-YouTube-Client-Name`/`X-YouTube-Client-Version` header | music_demo 只发 `X-Goog-Visitor-Id`/Origin/Content-Type（`api.rs:172-175`） |
| 5 | innertube 端点模板 | `_call_api` `_base.py:841` `https://{host}/youtubei/v1/{ep}`；host 来自 `INNERTUBE_HOST`（`_base.py:416`，web_music=music.youtube.com）；query `prettyPrint=false` `_base.py:847` | `api.rs:108-111` `https://music.youtube.com/youtubei/v1/search?key={}&prettyPrint=false`；`api.rs:166-169` `https://www.youtube.com/youtubei/v1/player?key={}&prettyPrint=false` | 结构一致；key 有无差异（见 #1） |
| 6 | player.js 端点 | 运行时构造：`_video.py:2146-2152` `_extract_player_url` 读 ytcfg `PLAYER_JS_URL`/`WEB_PLAYER_CONTEXT_CONFIGS.*.jsUrl`；URL 形如 `https://www.youtube.com/s/player/{8+hex-id}/{variant路径}`；变体路径表 `_PLAYER_JS_VARIANT_MAP` `_video.py:1889-1895`（main=`player_ias.vflset/en_US/base.js` 等 11 项） | **无对应**——music_demo 不下载 player.js（`player.rs:13-16` 注释明确「signature deciphering not yet implemented」） | 未来能力；变体路径表是纯数据可提取 |
| 7 | signatureTimestamp (sts) | 提取：`_video.py:2226-2259` `_extract_signature_timestamp` 对 player.js 内容 regex `(?:signatureTimestamp|sts)\s*:\s*(?P<sts>[0-9]{5})`（`_video.py:2252-2254`）；发送：`_generate_player_context` `_video.py:2688-2697` | `types.rs:52-55` `ContentPlayerPlaybackContext.signature_timestamp` **已定义但从未发送**（`api.rs:170` `playback_context: None`） | 字段已备好；提取逻辑（下载 JS + regex）缺失 |
| 8 | 签名算法（s 挑战） | `_video.py:3300-3337` `solve_js_challenges` 批量派发 `JsChallengeRequest`（`jsc/provider.py:36-55`）→ `jsc/_director.py` → Bun/Deno/Node/QuickJS 执行 `yt.solver.*.js`；应用：`_video.py:3554-3565`（`&signature=...`） | **无对应**（player.rs 过滤掉需要解签的格式：`format_requires_decipher` `api.rs:329-333` + `player.rs:60` filter） | C 级，见 §2 |
| 9 | n 挑战（URL `n` 参数） | `_video.py:3567-3577`（query `n`）、DASH/HLS manifest `/n/{challenge}`；求解同 #8 | **无对应**——ANDROID_VR 直链通常无 n 参数 | C 级 |
| 10 | po_token（GVS/Player/Subs 三类） | 策略常量：`_base.py:80-100` `WEB_PO_TOKEN_POLICIES`、各 client 的 `GVS_PO_TOKEN_POLICY`/`PLAYER_PO_TOKEN_POLICY`/`SUBS_PO_TOKEN_POLICY`（`_base.py:206-208`、`243-248`）；获取：`_video.py:2711-2745` `_get_config_po_token`、`_video.py:2747-2889` `fetch_po_token` → `pot/_director.py:257` → `pot/_builtin/webpo_cachespec.py`（BotGuard）；发送：player 请求 body `serviceIntegrityDimensions.poToken` `_video.py:2936-2937`、URL query `pot` `_video.py:3579-3580` | **无对应**——无任何 po_token 代码 | 策略标志是 A 级；求解机制是 C 级 |
| 11 | visitorData 机制 | 提取：`_base.py:919-928` `_extract_visitor_data`（源：ytcfg `VISITOR_DATA` → `INNERTUBE_CONTEXT.client.visitorData` → 响应 `responseContext.visitorData`，或 config arg）；发送：`X-Goog-Visitor-Id` header `_base.py:968` | `api.rs:29-90` `ensure_visitor_session`——抓 `https://www.youtube.com/` 首页，字符串扫描 `"VISITOR_DATA":"<base64>"`（`api.rs:57-75`），15min 缓存；`X-Goog-Visitor-Id` header `api.rs:172` | 机制等价已实现；提取源不同（yt-dlp 从已抓页面/响应取，music_demo 单独抓首页） |
| 12 | watch 页 `ytInitialPlayerResponse` 标记 | `_base.py:706-708` `_YT_INITIAL_PLAYER_RESPONSE_RE = r'ytInitialPlayerResponse\s*='`；解析用 `_search_json`（JS 字符串感知） | `api.rs:292` `let start_marker = "ytInitialPlayerResponse = ";` + 朴素找 `";}"` 截断（`api.rs:297-302`）——遇到 `;}` 出现在 JSON 字符串内部时会截错 | 标记常量可同步；解析逻辑 music_demo 更脆弱（B 级改进点） |
| 13 | `ytInitialData` 标记（搜索页/浏览页） | `_base.py:707` `_YT_INITIAL_DATA_RE` | **无对应**（search 走 API，`search.rs`） | 不需要 |
| 14 | 搜索端点 client 选择 | `_search.py:152` `_search_results(query, params, default_client='web_music')` → `_call_api` ep='search' | `api.rs:100-113` `InnertubeClient::search`（WEB_REMIX） | 一致 |
| 15 | 支持站点清单（自动发现新来源） | `supportedsites.md`（由 `devscripts/` 生成，遍历 `yt_dlp/extractor/` 的 `_VALID_URL`） | **无对应**（本批其它 agent 负责） | 属「新来源发现」范畴，与本映射正交 |

## 2. 可提取性评估（A=纯数据可自动提取，B=逻辑需人工适配，C=无法复用）

| 项 | 评级 | 理由 |
|---|---|---|
| #1 innertube API key | **C** | yt-dlp 已无常量（证据：commit `0e539617a` 删除；当前 `_call_api` 默认不发 key）。**替代方案**：music_demo 保留本地配置值（人工维护，漂移检测不覆盖），或仿 `youtube_live_chat.py:155` 在 `ensure_visitor_session`（`api.rs:29`，已抓首页）顺带解析 ytcfg 的 `INNERTUBE_API_KEY` 运行时提取——后者是 B 级逻辑移植 |
| #2/#3 client 配置（name/version/sdk/os/userAgent） | **A** | `INNERTUBE_CLIENTS` 是模块级纯字面量 dict（`_base.py:79-388`），AST 可完整解析提取。注意：`build_innertube_clients()`（`_base.py:415-436`）import 时会 setdefault `hl='en'`、`INNERTUBE_HOST` 等——提取脚本须用 AST 读字面量、**不要 import 执行** |
| #4 `INNERTUBE_CONTEXT_CLIENT_NAME` 数值 | **A** | 同 `INNERTUBE_CLIENTS` dict 内联整数 |
| #5 端点模板 | **A** | `_base.py:841` 的 f-string 模板 + `INNERTUBE_HOST` 值；`prettyPrint=false` 字面量。f-string 不是纯字面量，按已知模式（`https://{host}/youtubei/v1/{ep}`）正则匹配 |
| #6 player.js 变体路径表 | **A**（表）/ **B**（URL 构造） | `_PLAYER_JS_VARIANT_MAP`（`_video.py:1889-1895`）是字面量 dict → A；URL 的 player_id 需运行时从页面/iframe_api 提取（`_video.py:2154-2168`）→ B |
| #7 signatureTimestamp 提取 | **B** | 值是运行时从 player.js 内容 regex 提取（`_video.py:2252-2254`），regex 本身是常量（可提取），但「下载 JS + 缓存」是逻辑。music_demo `types.rs:54` 字段已就绪 |
| #8/#9 s/n 签名求解 | **C** | 依赖 jsc/ 架构：Python director + 外部 JS 引擎执行 `yt.solver.*.js`（vendored，`_info.py` HASHES 校验）。Rust 侧要么移植算法、要么内嵌 JS 引擎（quickjs-rs 等），跨语言无法直接复用。且 ANDROID_VR 直链方案下目前不触发 |
| #10 po_token 策略标志 | **A**（策略数据） | `GVS_PO_TOKEN_POLICY`/`PLAYER_PO_TOKEN_POLICY` 的 `required/recommended/not_required_for_premium/not_required_with_player_token` 布尔是字面量（`_base.py:206-208` 等）→ 可提取进 Rust 枚举/结构 |
| #10 po_token 求解 | **C** | BotGuard/WebPO HTTP challenge 交互（`pot/_builtin/webpo_cachespec.py`）+ `pot/_director.py` 编排是 Python 代码；无 API key 前提下 music_demo 未触发（yt-dlp 对 android_vr 的 GVS 策略 `not_required_with_player_token=True` 且 player token 非必需，`_base.py:243-248`） |
| #11 visitorData 提取 | **B** | 两边机制等价但实现不同（yt-dlp 从 ytcfg/responseContext 取，music_demo 首页正则）。无新增可提取数据；若要补齐 ytcfg 路径属人工移植 |
| #12 `ytInitialPlayerResponse` 标记 | **A**（正则串）/ **B**（解析逻辑） | regex 字面量 `_base.py:708` 可直接提取替换 music_demo 的 `start_marker`；但 yt-dlp 用 `_search_json` 做 JS 字符串感知解析，music_demo 的 `";}"` 截断是已知脆弱点，属 B 级改进 |
| #13/#14/#15 | A/A/— | #14 两端已一致无需同步；#15 归「新来源发现」提案 |

**汇总**：A 级 5 项（#2,#3,#4,#5,#6 表, #10 策略,#12 正则）；B 级 4 项（#6 构造,#7,#11,#12 解析）；C 级 3 项（#1 上游无源、#8,#9、#10 求解）。自动同步的收益集中在 **client 配置与策略标志**（#2/#3/#4/#10）——恰好是用户点名的「client 的 API key、context」中 context 的部分。

## 3. 同步载体设计

### 3.1 候选对比

| 候选 | 优点 | 缺点 |
|---|---|---|
| a) build.rs/codegen 构建期生成 | 编译期保证最新 | **构建期硬依赖 `../yt-dlp` 外部路径**（不在本仓库、CI 机器未必有）；`build.rs` 现仅 `tauri_build::build()`；上游 checkout 移动即破坏构建 |
| b) 外部 JSON 运行时加载 | 可热更、不重编译 | 运行时文件缺失/路径问题；Tauri 打包要处理资源嵌入；低频变动不值得运行时 IO |
| c) 生成补丁文件（git apply） | 走正常 review/提交流程 | 半自动，仍需人工合入；patch 与上游 commit 绑定脆弱 |

### 3.2 推荐：分层「快照 JSON + 提交式 codegen + 编译期常量」

- **同步脚本**：`scripts/sync_ytdlp.py`（提交进 music_demo 仓库）。功能：`--check`（漂移检测，§4）、`--update`（重生成）、`--verify`（§5）。**只在需要时手动/CI 触发**，不进 build.rs——构建期不依赖 `../yt-dlp`。
- **中间快照**：`scripts/yt_innertube_snapshot.json`（提交入库）——从 `../yt-dlp/yt_dlp/extractor/youtube/_base.py` 用 **Python `ast` 模块**解析 `INNERTUBE_CLIENTS` 字面量 dict（不 import 执行），归一化为 JSON。
- **生成物**：`src-tauri/src/extractor/youtube/generated.rs`（提交入库，`// @generated by scripts/sync_ytdlp.py — DO NOT EDIT`）——`api.rs` 引用其中的常量构造 `ClientContext`，替换现有内联字符串（`api.rs:104-105`、`api.rs:162-169`）。
- **运行时**：`include!`/`const` 编译进二进制，**不做运行时文件读取**。理由：这些值低频变动、必须与二进制版本一致（clientVersion 过期会导致 403，静默热更反而掩盖漂移）；编译期常量 + 生成头注释带上游 commit 号，可审计。

**开闭原则符合性**（`docs/architecture.md` 2.2）：改动全部收在 `extractor/youtube/` 内部——新增 `generated.rs`、`api.rs` 引用点、`types.rs` 不动或仅加 `From` 转换。`PlaybackService`/`MusicHandler`/resolver/前端零改动；同步机制本身（脚本 + 快照）是独立工具链，不进入核心管线。

### 3.3 示例格式

`scripts/yt_innertube_snapshot.json`（中间物）：
```json
{
  "source": { "repo": "yt-dlp", "commit": "fdcc954df", "date": "2026-07-23" },
  "api_key": null,
  "clients": {
    "web_music": {
      "client_name": "WEB_REMIX",
      "client_version": "1.20260707.12.00",
      "innertube_context_client_name": 67,
      "innertube_host": "music.youtube.com",
      "gvs_po_token_policy": { "https": { "required": true, "recommended": true, "not_required_for_premium": true, "not_required_with_player_token": false } }
    },
    "android_vr": {
      "client_name": "ANDROID_VR",
      "client_version": "1.65.10",
      "android_sdk_version": 32,
      "os_name": "Android",
      "os_version": "12L",
      "user_agent": "com.google.android.apps.youtube.vr.oculus/1.65.10 (Linux; U; Android 12L; eureka-user Build/SQ3A.220605.009.A1) gzip",
      "innertube_context_client_name": 28,
      "gvs_po_token_policy": { "https": { "required": true, "recommended": true, "not_required_for_premium": false, "not_required_with_player_token": true } }
    }
  }
}
```

`src-tauri/src/extractor/youtube/generated.rs`（生成物，示意）：
```rust
// @generated by scripts/sync_ytdlp.py — DO NOT EDIT
// source: yt-dlp fdcc954df (2026-07-23)

pub struct InnertubeClientConfig {
    pub client_name: &'static str,
    pub client_version: &'static str,
    pub android_sdk_version: Option<i64>,
    pub os_name: Option<&'static str>,
    pub os_version: Option<&'static str>,
    pub user_agent: Option<&'static str>,
    /// INNERTUBE_CONTEXT_CLIENT_NAME（X-YouTube-Client-Name header 数值）
    pub context_client_name: i64,
}

pub const WEB_MUSIC: InnertubeClientConfig = InnertubeClientConfig {
    client_name: "WEB_REMIX",
    client_version: "1.20260707.12.00",
    android_sdk_version: None, os_name: None, os_version: None, user_agent: None,
    context_client_name: 67,
};

pub const ANDROID_VR: InnertubeClientConfig = InnertubeClientConfig {
    client_name: "ANDROID_VR",
    client_version: "1.65.10",
    android_sdk_version: Some(32), os_name: Some("Android"), os_version: Some("12L"),
    user_agent: Some("com.google.android.apps.youtube.vr.oculus/1.65.10 ..."),
    context_client_name: 28,
};

/// 上游（yt-dlp）已删除硬编码 innertube key；本值由本地维护，漂移检测不覆盖。
pub const INNERTUBE_API_KEY: Option<&'static str> = Some("AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8");
```

同步脚本改动点（最小 diff）：`api.rs:104-105`、`api.rs:162-169` 改为从 `generated::WEB_MUSIC` / `generated::ANDROID_VR` 构造；`api.rs:7` 的 key 移到 `generated.rs` 并注释来源状态；`types.rs:12-27` `ClientContext` 增加 `#[serde(skip_serializing_if)]` 的 `user_agent` 已存在（`types.rs:18`）可直接填充。

## 4. 漂移检测设计

### 4.1 值级 diff（主检测）

`scripts/sync_ytdlp.py --check`：
1. 用 `ast.parse('../yt-dlp/yt_dlp/extractor/youtube/_base.py')` 定位 `INNERTUBE_CLIENTS` 赋值节点，`ast.literal_eval` 解出 dict（`build_innertube_clients()` 的 setdefault 变异不参与——AST 只读字面量）。
2. 归一化（只保留 music_demo 用到的子集：`clientName/clientVersion/androidSdkVersion/osName/osVersion/userAgent` + `INNERTUBE_CONTEXT_CLIENT_NAME` + `INNERTUBE_HOST` + 三项 PO policy 的布尔）→ 与 `scripts/yt_innertube_snapshot.json` diff。
3. 差异 → 打印 `old → new` 对照（如 `ANDROID_VR clientVersion: 1.65.10 → 1.66.0`）+ 退出码非 0（供 CI 红灯）。
4. 附加：`git -C ../yt-dlp rev-parse HEAD` 与快照 `source.commit` 比对——上游动过就提示跑 `--update`。

实现要点：`_base.py:79-388` 的 dict 是纯字面量，`ast.literal_eval` 对 `StreamingProtocol.HTTPS` 这类枚举键会失败——提取脚本需对 `GVS_PO_TOKEN_POLICY` 键做已知白名单转换（`StreamingProtocol.HTTPS/DASH/HLS` → 字符串），或策略键直接按 `dict(dict)` 顺序映射。`GvsPoTokenPolicy(...)`/`PlayerPoTokenPolicy(...)` 是 keyword-arg 构造调用，用 AST 节点逐字段取即可，不需要执行。

### 4.2 提交日志关键字（辅助）

```bash
git -C ../yt-dlp log --oneline -30 -- yt_dlp/extractor/youtube/_base.py yt_dlp/extractor/youtube/_video.py
# 再按关键字过滤：client|innertube|player|po.?token|signature|visitor|api key
```
触发告警后人工看 commit 是否影响 music_demo 使用面。适合 B/C 级项（sts、签名、po_token 机制变化不产生值 diff，但可能有行为变化）。

### 4.3 锚点字面量检测（B/C 级预警）

对 B/C 级的关键字面量做**存在性**检测（值变了不算失败、消失了才算）：
- `_video.py` 中 `(?:signatureTimestamp|sts)\s*:` regex 存在；
- `_PLAYER_JS_VARIANT_MAP` 中 `main` 条目存在；
- `jsc/_builtin/vendor/_info.py` 存在（签名求解架构没被删）。
任何锚点消失 → 红色告警「上游重构，需人工评估移植」。

### 4.4 触发方式

- CI job（music_demo 侧，`--check` 仅读 ../yt-dlp 不写仓库）；
- 本地 `make sync-check` 类别名；
- 上游 release 提醒后人工 `--update` + review + 提交。

## 5. 验证方案

### 5.1 现状盘点（已核实）

- **离线单测**（全部不触网，`cargo test` 覆盖）：`youtube/mod.rs:42-85` `extract_video_id` ×5；`youtube/api.rs:354-368` `parse_duration_to_ms` ×1；`youtube/player.rs:148-165` `is_rodio_playable` ×2。
- **网络 smoke 通道**：`cli/src/main.rs` —— `yt-search`（`main.rs:105`）、`yt`（`main.rs:171` get_manifest、`main.rs:298` 列表）、`main.rs:201` validate_url。CLI 直连 `ExtractorContext`，不经播放管线（`docs/architecture.md` §3.3）。
- 无录制 fixture、无 youtube 网络集成测试。

### 5.2 同步后验证清单

1. **离线契约测试（必做，CI 可跑）**：`api.rs` 新增 `#[cfg(test)]` 断言——`search()`/`player()` 构造的请求体序列化后 `context.client.clientName/clientVersion` 等于 `generated::WEB_MUSIC`/`generated::ANDROID_VR` 字段（用 `serde_json::to_value` 断言，不触网）。这直接防「手写字符串与生成常量漂移」——比 diff 脚本更早拦截。
2. **回归**：`cargo test` 全绿（现有 3 个 youtube 单测 + 全仓）。
3. **网络 smoke（需要外网）**：`cargo run -p music-cli -- yt <video_id>` 返回非空 `streams` 且首个流 `validate_url` 200/206（`main.rs:201` 已有该检查路径）；`yt-search` 返回结果。`scripts/sync_ytdlp.py --verify` 可把这两步串起来。
4. **行为等价性对照（可选，有网络时）**：同 video_id 下 yt-dlp（`python -m yt_dlp -J --skip-download`）与 music-cli 各自拿到的音频流 URL 前缀一致（googlevideo.com），确认 client 配置同步后协议行为未变。
5. **失败模式预检**：若 `--check` 检出 clientVersion 漂移而尚未同步，预期现象是 player 端点返回 403/`LOGIN_REQUIRED`——`check_playability`（`api.rs:245-263`）与 `check_api_error`（`api.rs:265-274`）已有对应错误映射，可作为人工确认信号。

## 6. 结论与落地顺序

1. **立即可行（纯数据）**：把 `INNERTUBE_CLIENTS` 中 web_music/android_vr 的 client 配置 + `INNERTUBE_CONTEXT_CLIENT_NAME` + PO policy 布尔提取为快照 JSON → 生成 `generated.rs` → `api.rs` 改用常量。这是本方案的主体收益。
2. **API key**：yt-dlp 无源（C 级）。推荐保留本地常量并注释上游状态；后续若 key 失效，升级方案是 `ensure_visitor_session`（`api.rs:29`，已抓首页）顺带解析页面 ytcfg 的 `INNERTUBE_API_KEY`（仿 `downloader/youtube_live_chat.py:155`），属 B 级移植、可作独立后续任务。
3. **签名/po_token（B/C 级）**：不做自动同步；漂移检测的锚点 + 日志关键字机制负责「上游变了提醒人工」。ANDROID_VR 直链方案下这两块当前不触发，优先级最低。
4. **顺带修复点**：`api.rs:292-302` 的 `ytInitialPlayerResponse` 朴素截断可改用 yt-dlp 的正则常量（A 级提取）+ `_search_json` 式解析（B 级），与同步机制同批落地。

> 证据索引：`yt-dlp@fdcc954df` `_base.py:79-388,415-436,706-708,830-848,919-975,991-1029`；`_video.py:1889-1895,2091-2099,2146-2259,2688-2697,2911-2937,3300-3337,3554-3577`；`jsc/provider.py:36-55`；`pot/_director.py:257`；`downloader/youtube_live_chat.py:155`；git commit `0e539617a`（key 移除）。music_demo：`api.rs:7,29-90,100-175,245-302,329-333`；`types.rs:12-27,52-55`；`player.rs:13-16,60`；`mod.rs:42-85`；`cli/src/main.rs:105,171,201,298`。
