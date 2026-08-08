# 架构决策提案：yt-dlp 同步方案 A/B/C 评估

> 状态：brainstorm 提案（未实施，不改任何代码）
> 作者：agent `BridgeVsStatic`，2026-08-06
> 关联：`docs/proposals/upstream-sync-workflow.md`（工作流层）、`docs/proposals/ytdlp-youtube-field-sync.md`（字段层）、`docs/proposals/ytdlp-new-source-sync.md`（新来源层）

## 0. 结论速览（TL;DR）

**推荐 C（混合）**：保留 Rust 手写 extractor 作为 YouTube/Bilibili 主力快路径，新增「yt-dlp 桥接」作为三层用途：① 任意 URL 直粘播放（泛化现有 ParseUrl）；② 一切无原生 extractor 的来源；③ YouTube/Bilibili 原生解析失败的兜底。

核心理由（每条有证据，见正文）：
- 桥接与现有 spool 流式管线**零冲突**：`PlaybackManifest` 的 `url + headers` 契约与 yt-dlp `-J` 的 `formats[].url + http_headers` **一一对应**（service.rs:353-355 实测读码）；
- 纯 B 的代价集中在打包与进程管理（Tauri v2 核心无 sidecar API，需新增 `tauri-plugin-shell` + `externalBin` + macOS 签名），C 让这些代价只为「长尾来源 + 兜底」支付；
- 纯 A 的维护负担已被量化：上游一年 **391 个 extractor 提交 / 97 个 youtube 提交 / 35 个新来源文件 / 20 个 release**，且我们 Rust YouTube extractor **明确不支持签名解密**（player.rs:12-13 注释），ciphered-only 视频直接播放失败——桥接兜底直接补上这个已知缺口。

## 1. 证据基线（三条路线共用的事实）

### 1.1 现状：Rust 自研 extractor 的已知局限（读码）

| 事实 | 证据 |
|---|---|
| YouTube extractor **不实现签名解密**（s/n-sig），ciphered 格式被排除 | `extractor/youtube/player.rs:12-13` 注释 *"Some formats may have ciphered URLs requiring signature deciphering (not yet implemented). Those formats are excluded"*；`player.rs:82-86` 报错 `found N audio format(s), but none are directly playable by rodio (M require signature deciphering)` |
| YouTube 播放格式被过滤到 rodio 可解码子集（AAC/MP4、MP3；**无 Opus**） | `player.rs:46-52` 注释 *"does not ship an Opus decoder. Do not hand WebM/Opus URLs"*；`is_rodio_playable()`（player.rs:114+） |
| Bilibili 走官方 API + WBI 签名 + buvid3 cookie，分页搜索 | `extractor/bilibili/search.rs:9-57`（`api.bilibili.com/x/web-interface/search/type` + `encode_wbi` + `ensure_cookie`） |
| YouTube 搜索是 YouTube Music InnerTube（Songs/Albums/Videos section 参数） | `extractor/youtube/search.rs:10-34`（`music.youtube.com/youtubei/v1/search`，section 参数注释在 16-18 行） |
| 前端已有 URL 粘贴入口，但目前**只接受微信公众号链接** | `src/components/ParseUrl.tsx:24-26` `validateWechatUrl` 限定 `mp.weixin.qq.com`，invoke `parse_track_from_wx` |
| 播放管线消费 `PlaybackManifest{streams: Vec<AudioStream{url,mime_type,bitrate,codec,content_length}>, headers: HashMap, expires_at}`，下载 = `reqwest.get(url).headers(manifest.headers)` 按 bitrate 降序尝试候选 | `extractor/model.rs:27-42`；`playback/service.rs:316-355, 658-668` |

### 1.2 上游 yt-dlp 实测（本机 /Users/wbyuan/proj/yt-dlp，2026.07.04，Python ≥3.10）

| 测量项 | 结果 | 命令 |
|---|---|---|
| 系统 python3 版本 | **3.9，不满足 yt-dlp `requires-python = ">=3.10"`**（pyproject.toml） | `python3 -V`；`uv run python -m yt_dlp --version` 首次报错 |
| 冷启动（首次 `uv run`，装依赖） | 5.2s（一次性） | `time uv run python -m yt_dlp --version` |
| 温启动（venv 直跑） | **~85–123ms** | `time .venv/bin/python -m yt_dlp --version` |
| 完整 YouTube 解析（真实 URL） | **2.9s wall**，49 个 formats **全部带签名直链 url + 每格式 http_headers** + abr/ext/protocol/filesize | `.venv/bin/python -m yt_dlp --dump-single-json --no-warnings --skip-download https://www.youtube.com/watch?v=dQw4w9WgXcQ` |
| YouTube 搜索（flat） | **1.8s**，3 条结果含 id/title/uploader/duration/**thumbnails**/url | `yt-dlp --dump-single-json --flat-playlist "ytsearch3:never gonna give you up"` |
| 搜索类 extractor | **10 个**：ytsearch、bilisearch、scsearch(SoundCloud)、nicosearch、gvsearch、yvsearch、rkfnsearch、prxseries、prxstories、nicosearchdate | `gen_extractor_classes()` 过滤 `_SEARCH_KEY` |
| bilisearch 存在性 | 存在（BiliBiliSearchIE）；本机实测 412（bilibili 对当前出口的反爬），错误发生在网络层，playlist 包装解析正常 | `yt-dlp "bilisearch2:周杰伦"` → `HTTP Error 412` |
| 规模 | **1751 个 extractor 类 / 942 个 extractor 文件 / supportedsites.md 1738 行** | `gen_extractor_classes()`；`ls yt_dlp/extractor/` |
| 更新节奏 | 近一年 **20 个 tag / 562 提交**；youtube extractor **97 提交/年**、bilibili 5 提交/年、新增 extractor 文件 35 个/年 | `git log --since="1 year ago"` 系列 |

### 1.3 打包/运行时事实（读码）

| 事实 | 证据 |
|---|---|
| `tauri.conf.json` bundle：**无 `externalBin`**（无 sidecar），targets 仅 `["deb","rpm"]` | `src-tauri/tauri.conf.json` `bundle` 节 |
| CI 只构建 Linux + Windows，**无 macOS job**（macOS 产物为本地构建） | `.github/workflows/build.yml`（test/build-windows/build-linux）、`release.yml` |
| Cargo.toml：**无 pyo3、无 tauri-plugin-shell**；`tauri = { version = "~2.11.5", features = [] }` | `src-tauri/Cargo.toml` |
| **Tauri v2 核心已无 `new_sidecar`**（process.rs 只有 current_binary/restart），sidecar 必须经 `tauri-plugin-shell` | `~/.cargo/registry/src/*/tauri-2.11.5/src/process.rs` 全文 grep；v2 sidecar 属 shell 插件能力（[Tauri v2 Sidecar 文档](https://v2.tauri.app/develop/sidecar/)）[INFERENCE: 官方文档结论，本地未装该插件] |
| yt-dlp **官方就用 pyinstaller 出 mac 二进制**（CI 产出 `yt-dlp_macos`），仓库自带 `bundle/pyinstaller.py` + `pyproject.toml [project.entry-points.pyinstaller40] hook-dirs` | `yt-dlp/.github/workflows/build.yml:27,68-69,301-302`；`bundle/pyinstaller.py:11,46-49` |
| ResolverRegistry 按 SourceKind 存 `Vec<Arc<dyn PlaybackResolver>>`，`accepts()` 过滤，但**当前逻辑取第一个 accept 就返回，失败不尝试下一个** | `playback/resolver.rs:44-56`（`for resolver … { if accepts { return resolve } }`） |
| 「URL 型 SourceRef、纯 resolver 无搜索」的 adapter 先例已存在（WechatResolver：`SourceRef::Wechat{url}`、`MetaValue::Wechat`） | `playback/resolver.rs` 注册表；`types.rs:16-45` `TrackMeta` |

## 2. B 路线技术可行性（逐条）

### 2.1 子进程 `-J` 的 JSON 能否覆盖我们的字段 —— **能，一一对应**

实测 dQw4w9WgXcQ 的 `-J` 输出：
- 顶层：`title/duration/uploader/thumbnail/availability/playable_in_embed/formats[]`（49 个）；
- 每 format：`url`（**已签名直链**）、`http_headers`（`User-Agent/Accept/Accept-Language/Sec-Fetch-Mode`）、`abr`、`ext`、`protocol`、`vcodec/acodec`、`mime_type`、`filesize`；
- 音频格式实样：`249/250/251`（opus/webm，abr 46/61/129）→ 桥接需**过滤掉**（rodio 无 Opus，见 1.1），选 `mp4a/mp3`（如 format 140 m4a）。

→ 映射到 `PlaybackManifest`：`streams[i] = AudioStream{url: f.url, mime_type: f.mime_type, bitrate: f.abr, codec: f.acodec, content_length: f.filesize}`，`headers` 取选中流的 `http_headers`。**service.rs 的 spool 下载、bitrate 降序候选、缓存提交全部零改动**（service.rs:316-355 逐行比对）。签名/PO-Token/n-sig 的逆向全部由 yt-dlp 内部处理，对我们透明。

### 2.2 性能与常驻方案

- 实测：搜索 **1.8s**、播放解析 **2.9s**（含网络与签名 JS；进程启动仅 ~100ms，占比小）；
- 每次播放只在**缓存未命中**时产生一次解析开销（service.rs 缓存命中走 `TrackSource::File`，不再调 resolver，service.rs:27-31）；
- 常驻方案可行但收益有限：Python 守护进程（stdin/stdout JSON-RPC）只省掉 ~100ms 启动；收益低于进程管理复杂度。**推荐每请求一个 `tokio::process::Command` 子进程**（Rust 侧异步 spawn + 超时 + 取消），简单可控。

### 2.3 打包现实

- 系统 Python 3.9 **不可用**（yt-dlp 需 ≥3.10，实测 ImportError），「依赖系统 python」路线排除；
- **pyinstaller 单文件 sidecar 是已验证路线**：yt-dlp 官方 mac 二进制即 pyinstaller 产物（1.3），pyproject 自带 hook；Tauri 侧需：① 新增 `tauri-plugin-shell` 依赖；② `tauri.conf.json` 加 `bundle.externalBin: ["binaries/yt-dlp"]`（按平台 triple 命名 `yt-dlp-x86_64-apple-darwin` 等）；③ macOS 侧 sidecar 需参与签名；④ 体积增加（pyinstaller 单文件约数十 MB 量级）[INFERENCE: 具体体积需打包 spike 验证]。改造量：中等，一次性。

### 2.4 「更新问题自动消失」——量化卖点

| 路线 | 年度同步负担（近一年 git 实测） |
|---|---|
| A（手写移植） | 391 extractor 提交 + 20 release 需人工评估/移植；**97 个 youtube 提交**（n-sig/po-token/player-client 军备竞赛）——我们的 player.rs 已因此落后（不支持解密） |
| B/C 的桥接侧 | 同步 = **替换一个自包含的 yt-dlp 二进制/源码目录**（上游自带测试，20 release/年可选跟进，通常只需跟随），协议级改动 0 行 Rust |

## 3. B 路线的代价

### 3.1 与 spool 流式的冲突 —— **不冲突，但有一个前提**

- `docs/architecture.md 4.2` 的 spool 是「HTTP body 边写边读」（service.rs:273-473：reqwest GET → 临时文件 → `BlockingSpoolReader`）。yt-dlp 走 `download=False`/`-J` 只产出**直链**，下载仍由 reqwest 完成 → 流式保留；
- 前提：**桥接不得调用 yt-dlp 的下载路径**（`ydl.download()`），否则流式丢失。用 `-J --skip-download`（或 Python API `extract_info(url, download=False)` + `sanitize_info`）天然满足（README.md:2063-2077 明确建议 `-J/--print` 作机器接口）；
- 次要约束：HLS（m3u8）格式 spool 也播不了（rodio 无 HLS 解码）——与现状一致（原生 extractor 也只给直链），不构成降级。

### 3.2 运行时风险

- Python 崩溃/挂起：需超时 + 取消 + stderr 采集（tokio 子进程标准能力）；状态收敛在子进程内，**不污染 SSOT**（崩溃 = 该次 resolve 失败，走现有 PlaybackError 路径，architecture.md 4.2 顺序约束 3 已有失败语义）；
- 反爬影响：YouTube 反爬针对的是请求模式（n-sig/po-token/visitor_data），yt-dlp 内部处理，对子进程方案无额外影响；实测 2.9s 解析成功即证；
- 真风险是**版本漂移**：用户侧 yt-dlp 一旦停止更新，长尾来源会逐渐失效——因此 C 方案下桥接是「覆盖面」而非「主链路」。

### 3.3 搜索降级评估

- yt-dlp flat 搜索（实测 1.8s）给出 title/uploader/duration/thumbnails/url，**足够填满 `TrackView{title,artist,cover_url,duration,id}`**（types.rs:26-31）；
- 但相对原生降级明显：YouTube Music 的 Songs/Albums/Videos section 语义丢失（原生 search.rs:16-18）；Bilibili 官方 API 的播放量/弹幕/分页丢失（原生 search.rs:9-57）；flat 条目元数据薄、无 `view_count` 等；
- 结论：**搜索保持原生优先，桥接搜索只用于「无原生搜索的来源」**（如 SoundCloud 走 `scsearch:`）。

## 4. 决策矩阵

| 维度 | A 纯静态同步 | B 纯桥接 | C 混合（推荐） |
|---|---|---|---|
| 开发成本 | 低（现状延续） | 中高（进程管理、JSON 解析、打包、签名、失败语义） | 中（先做 URL 桥接 + 兜底，复用现有 adapter 骨架） |
| 维护成本 | **高**：391 extractor 提交/年、97 youtube 提交/年；n-sig/po-token 军备竞赛（我们已落后：不支持解密） | **低**：更新 = 换 vendored yt-dlp | **低-中**：两个主力的协议级移植仍存在，但失败时兜底屏蔽；长尾零维护 |
| 新来源覆盖速度 | 慢（每个来源手写：extractor + adapter + 测试，extension-guide.md 全流程） | **即时**（1751 extractor 全可用，bilisearch/scsearch 等 10 个搜索类） | 即时（桥接兜底一切非原生来源） |
| 播放体验（流式） | 原生快路径；但 ciphered-only 视频**直接失败** | 直链 + spool 流式保留；首播 +2.9s 解析延迟（之后缓存命中） | **最好**：原生快路径，失败自动降级桥接 |
| 打包复杂度 | 无变化 | **高**：pyinstaller + shell 插件 + externalBin + macOS 签名 + 体积（当前 bundle 无 externalBin，CI 无 macOS job） | = B 的打包成本（只付一次） |
| 离线/隐私 | 最优（单进程、无额外运行时） | 较差（捆绑 Python 运行时、常驻/反复起进程、崩溃面更大） | 中间（桥接仅在需要时起进程） |

## 5. 推荐：C（混合）与桥接优先判据

**推荐 C**。理由链：桥接与 spool 契约 1:1（2.1）→ 技术可行；纯 B 的打包/进程成本只为长尾支付不划算（4）；纯 A 已在 YouTube 签名解密上失守（1.1）且年同步负担被量化（2.4）→ 桥接兜底是低成本止血。

**桥接优先用于以下来源类型（判据）**：
1. **URL 直粘播放**（第一优先级）：现有 ParseUrl 只收微信公众号（ParseUrl.tsx:24-26），泛化为「任意 URL → 桥接解析 → 播放」。这是 2000+ extractor 唯一能发挥的场景，且纯 resolver（无搜索）完全契合 `SourceKind::Generic` + `SourceRef::Url{v}` 模式（1.3 Wechat 先例）；
2. **无原生 extractor 的来源**（SoundCloud、Nico 等，或用户后续贴的任何 URL）：桥接 = 该来源的 adapter，不写协议代码；
3. **YouTube/Bilibili 原生解析失败兜底**：同一 `SourceKind` 下注册第二个 resolver（原生失败 → 桥接重试）。需要把 `ResolverRegistry::resolve` 改为「失败尝试下一个」（resolver.rs:44-56，~5 行改动 + 测试）；若不想动核心，可退化为 adapter 内部自兜底（「来源差异停留在 adapter 边界」也符合 extension-guide.md §6）。
4. **搜索桥接**：仅当来源无原生搜索时启用（scsearch/bilisearch 等 10 个前缀），避免 YouTube Music section 体验降级（3.3）。

**不做的事**：不把 YouTube/Bilibili 主链路换成桥接（首播 +2.9s、打包依赖、搜索降级都不值）；不引入 pyo3 内嵌（Python 解释器进 Rust 进程，崩溃面与复杂度双升，收益仅 ~100ms）。

## 6. 演进路径（纯 A → 推荐 C）

| 步骤 | 内容 | 交付物 | 风险/缓解 |
|---|---|---|---|
| **S1 桥接内核（dev 环境）** | 新增 `src-tauri/src/ytdlp_bridge/`：`spawn -J` 子进程（`tokio::process::Command`，`--no-warnings --skip-download --no-playlist --no-config`，超时+取消）、JSON→`PlaybackManifest` 映射（含 rodio 过滤：`acodec mp4a/mp3`，复用 `is_rodio_playable` 语义）；新增 `SourceKind::Generic` + `SourceRef::Url` + `TrackId` 前缀 + `track_to_entry` arm + `TrackMeta`（extension-guide.md §2 清单） | dev 构建可解析任意 URL；单测（manifest 映射/过滤/失败路径）+ 3 个真实站点 smoke | Python 依赖只在 dev 机（用 yt-dlp repo venv）；不触碰打包 |
| **S2 桥接 search adapter** | `SearchProvider` 实现：keyword → `ytsearch:/bilisearch:/scsearch:` 前缀分发（仅注册给无原生搜索的来源）；flat entries → `TrackView`（实测字段齐备） | `search_music` 覆盖新增来源，注册表零核心改动 | flat 元数据薄 → 仅作为补充来源，不替换原生搜索 |
| **S3 兜底链** | `ResolverRegistry::resolve` 改为失败续试下一个；Youtube/Bilibili 各自注册原生 + 桥接两个 resolver | 原生失败（如 ciphered-only、po-token 缺失）自动降级，播放不中断 | 核心文件小改动 → 补 registry 测试（extension-guide.md §5 测试要求） |
| **S4 sidecar 打包** | pyinstaller 构建 yt-dlp 二进制（官方脚本）+ `tauri-plugin-shell` + `externalBin` + macOS 签名 + 体积验证；bridge 路径切换为 sidecar 发现（开发时仍可用 venv） | 发布产物离线自包含，任意 URL 可播 | 体积/签名/CI（当前无 macOS job）→ 先本地 spike，再补 CI |
| **S5 版本跟随策略** | vendored yt-dlp 版本随应用 release 固定（记录在 README/依赖清单），跟随上游 20 release/年节奏按需升级；升级 = 替换二进制 + 回归 smoke | 可复现的更新流程；`ytdlp_bridge` 稳定性回归 | yt-dlp API/输出格式漂移 → S1 的映射层集中隔离 + 契约测试锁定字段 |

每步独立可交付、可回退；S1-S3 不依赖打包，S4 不改变架构。

## 7. 附录：关键实测命令

```bash
cd /Users/wbyuan/proj/yt-dlp
time .venv/bin/python -m yt_dlp --version                          # 85–123ms 温启动
time .venv/bin/python -m yt_dlp --dump-single-json --no-warnings --skip-download "https://www.youtube.com/watch?v=dQw4w9WgXcQ"  # 2.9s，49 formats 全带签名 url + http_headers
time .venv/bin/python -m yt_dlp --dump-single-json --flat-playlist --no-warnings --skip-download "ytsearch3:never gonna give you up"  # 1.8s
git log --since="1 year ago" --oneline -- yt_dlp/extractor/youtube/ | wc -l   # 97
git log --since="1 year ago" --diff-filter=A --name-only -- yt_dlp/extractor/ | grep -c '\.py'  # 35 新来源
```

> 注：以上实测均在 /Users/wbyuan/proj/yt-dlp 的 uv 管理 venv 内完成（系统 python3 为 3.9 不可用）；bilisearch 的网络 412 属于本机出口反爬，非解析层问题。
