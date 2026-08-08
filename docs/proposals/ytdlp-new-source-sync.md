# Proposal: yt-dlp 新来源自动发现与接入方案

> 状态：brainstorm 提案（未实施，不改任何代码）
> 作者：agent `NewSourceDiscovery`，2026-08-06
> 关联文档：`docs/extension-guide.md`、`docs/architecture.md`、`docs/contracts.md`
> 上游资源：`/Users/wbyuan/proj/yt-dlp`（git clone，origin = github.com/yt-dlp/yt-dlp）

---

## 0. 结论速览（TL;DR）

- **候选清单生成器：完全可行且成本极低**。不解析 `supportedsites.md`，而是直接调用 yt-dlp 的 Python 枚举 API（`list_extractor_classes()`，与上游 `make_supportedsites.py` 同源），按关键词评分过滤出音乐/电台/播客候选，产出 JSON 清单。实测全库 1731 条 extractor 中，严格关键词过滤得到 **~87 个 base 站点**，precision 尚可（约 80%），边界站点进入人工审核列表。
- **五步接入的自动化**：步骤 3/4/5（标识层、runtime 注册、前缀表）**可 100% 自动**；步骤 2（adapter）**可 90% 自动**（模板 + 唯一变量是 `SourceRef` 字段名）；步骤 1（extractor 协议逆向）**必须人工**，无法自动翻译 yt-dlp 的 Python 逆向代码。测试骨架可自动生成，smoke 测试留人工/CI 夜间。
- **通用代理方案（yt-dlp 子进程）**：**不推荐作为默认主路线**。搜索链路被证实是**串行枚举**（`public.rs:search_music`），代理化后 "all" 搜索 = N 个串行子进程查询（每个 2–8s）；播放链路（symphonia）只能吃**单文件音频**，大量电台/播客只提供 HLS-AAC/DASH，代理必须自带 format 过滤层；且音乐站点里**只有 soundcloud 有 `_SEARCH_KEY`**，Bandcamp/QQ音乐/网易云等 yt-dlp 本身也无法搜索，"新来源自动获得搜索"打折。但代理方案作为**开发期 oracle（差分测试）**与**生产期"URL 粘贴兜底 resolver"**有明确价值。
- **推荐路线**：短期落地生成器 + 变更报告 + issue 闭环（零代码改动）；中期落地 adapter 代码生成器（marker 块机制）把人工压到"只写 extractor + 只勾选审核项"；长期保留 yt-dlp sidecar 作为决策选项（桌面可行，移动端不可行）。

---

## 1. 现状与关键事实（研究结论，含证据）

### 1.1 music_demo 侧：新来源接入 = 5 步 + 2 个隐藏步骤

`docs/extension-guide.md` 给出的接入路径，落到具体符号：

| # | 步骤 | 具体位置 | 变量部分 |
|---|---|---|---|
| 1 | extractor 协议代码 | `src-tauri/src/extractor/<source>/{search,player}.rs` + `mod.rs` 声明 | **全部**（协议逆向） |
| 2 | adapter | `src-tauri/src/playback/<source>.rs`（`SearchProvider` + `PlaybackResolver`） | `SourceRef::Xxx { <field> }` 字段名 |
| 3 | 标识层 | `playback/model.rs`：`SourceKind` 枚举 + `as_str()`、`TrackId::Display/FromStr`、`SourceRef` 变体 + `kind()`；`types.rs`：`TrackMeta::from_source_ref/to_source_ref` | 枚举名、前缀、字段名 |
| 4 | 注册 | `playback/runtime.rs` `BackendRuntime::new`（两个 registry 各一行） | 类型名 |
| 5 | 前缀表 | `playback/search.rs` `track_to_entry` 的 match arm | 前缀 |
| 6 | 测试 | extension-guide §5：contract 测试 + smoke | 断言值 |
| 7 | DB 兼容 | `types.rs::MetaValue::Extractor(String)` 存 `"<prefix>:<id>"` | **无需新变体**，只需 `to_source_ref` 的 `strip_prefix` 分支 |

**两个关键发现（影响后续设计）：**

1. **`search_music` 是串行枚举**（`public.rs:47-60`）：对每个 `SourceKind` 依次 `await`。这直接决定了"代理方案 + search-all"的延迟 = N × 单次子进程查询。当前 2 个来源尚可，代理化到 10+ 站点不可接受。
2. **播放链路只能播单文件音频**：`PlaybackService` → spool（HTTP body 边写边读）→ rodio/symphonia `Decoder::try_from`。**HLS m3u8 / DASH fMP4 不可播**（m3u8 下载下来是文本，解码失败）。任何"用 yt-dlp 拿流"的方案都必须自己过滤 `formats`（`protocol ∈ {http,https}` + `vcodec == none` + `ext ∈ 音频集合`），等于在 Rust 侧重写半套 format 选择逻辑。

### 1.2 关键文件证据

- `public.rs:29-60`：`search_music` 顺序 for 循环，`source=="all"` 时枚举 `SearchRegistry::sources()`。
- `playback/model.rs`：`SourceKind{Youtube,Bilibili,Wechat}`；`TrackId::FromStr` 只认 `yt:`/`bili:` 前缀；`SourceRef::Legacy{meta}` 的 `kind()` 把 `MetaValue::Extractor(_)` 一律映射为 `Youtube`（既有怪癖，新来源**不要动**，只影响 legacy 数据路径）。
- `types.rs`：`MetaValue::Extractor(String)` —— 新来源**不需要**新增变体，DB schema 不变。
- `extractor/model.rs`：`Track{id,title,artists,album,duration_ms,artwork}`、`AudioStream{url,mime_type,bitrate,codec,content_length}`、`PlaybackManifest{streams,headers,expires_at}` —— 统一的、来源无关的业务模型，新来源 adapter 的转换逻辑几乎是机械映射。
- `tauri.conf.json`：bundle targets = `["deb","rpm"]`；Cargo 无 `tauri-plugin-shell`（代理方案需评估 sidecar 机制）。
- `.github/workflows/`：已有 `build.yml`、`release.yml`，CI 骨架现成。
- **`docs/spec-yt-dlp.md` 已被删除**（`git status` 显示 `D`）：该文档（cross-platform extractor spec）明确写过"**系统不直接依赖 yt-dlp binary**"，yt-dlp 仅作为开发/CI 的 reference + oracle，其 §8 设计了一个"定期跟踪 yt-dlp 变化 → 生成 oracle JSON → 与 Rust extractor 差分"的工作流，**从未落地**。本提案第 6 节会以轻量形式复活它。删除意味着团队方向已转向"自研 Rust extractor"，代理方案是与此先例相悖的——评估时必须正视。

### 1.3 yt-dlp 侧：可编程枚举，不需要解析 markdown

- `devscripts/make_supportedsites.py`（全文 30 行）就是枚举器：`from yt_dlp.extractor import list_extractor_classes` → 过滤 `IE_DESC is not False` → 排除 `GenericIE` → 按 `IE_NAME` 排序 → 每个 `ie.description()` 一行。**`supportedsites.md` 只是这份枚举的渲染**。
- `yt_dlp/extractor/__init__.py`：`gen_extractor_classes()` → `import_extractors()` 导入 `_extractors.py`（941 个文件、2475 行 import 列表）→ `_extractors_context`。
- `yt_dlp/extractor/common.py` 每个 extractor 类可编程读取的属性：
  - `IE_NAME`（classproperty = 类名去掉 `IE` 后缀）
  - `_VALID_URL`（正则 / `False`=embed-only / SearchInfoExtractor 由 `_SEARCH_KEY` 派生）
  - `IE_DESC`（`False` = 隐藏条目）
  - `_NETRC_MACHINE`（登录需求信号）、`_WORKING`（损坏标记，对应 supportedsites.md 的 `(**Currently broken**)`）
  - `SEARCH_KEY`（`SearchInfoExtractor._SEARCH_KEY`，如 `scsearch`/`ytsearch`/`bilisearch`）
  - `description(markdown=True)` 直接产出站点描述文本
- **搜索形态**：`SearchInfoExtractor`（common.py:4125）`_VALID_URL = r'{_SEARCH_KEY}(|N|all):{query}'`；`SoundcloudSearchIE._get_n_results` 返回 `playlist_result(entries)` → CLI 侧 `yt-dlp -J "scsearch10:query"` 输出单行 playlist JSON（`entries[]`）。**全库 `_SEARCH_KEY` 全集只有 10 个**：`ytsearch, scsearch, bilisearch, gvsearch, nicosearch, nicosearchdate, prxstories, prxseries, rkfnsearch, yvsearch` —— 音乐站点里**只有 soundcloud 一个**有搜索 extractor。
- **环境约束**：本地 `python3` 是 3.9.6，而 yt-dlp `requires-python = ">=3.10"`（pyproject.toml:18）→ 生成器必须跑在 venv/uv 里（设计见 §2.4）。yt-dlp 核心 `dependencies = []`（无硬依赖），包体 11MB / 1045 个 py 文件。
- 代表性音乐 extractor 规模：`qqmusic.py` 492 行、`bandcamp.py` 544 行、`kuwo.py` 352 行、`youtube/_base.py` 1349 行 —— **几百行 Python 逆向逻辑，自动翻译成 Rust 不可行**（可行性边界 §4）。

---

## 2. 候选清单生成器设计

### 2.1 设计原则

1. **直接调用 yt-dlp 枚举 API，不解析 `supportedsites.md`**（与上游同源，天然免疫格式漂移，且能拿到 markdown 里没有的 `_WORKING`/`SEARCH_KEY`/`_NETRC_MACHINE` 等结构化字段）。
2. **双级过滤**：自动关键词评分（高召回）→ 候选清单 + 边界清单，**人工只做 yes/no 勾选**（高精度），不做任何格式整理。
3. **输出与代码零耦合**：清单是 `sync/out/*.json` + 变更报告 md；接入决策（prefix、enum 名）落在一个**人工维护的注册表** `sync/sources.json`（见 §3.3），不在代码里反推。
4. **幂等可重跑**：`git fetch` 上游后重跑，只输出增量 diff。

### 2.2 过滤与评分

对每个 extractor 计算 `music_score`：

- **强正信号**（命中即入围）：`music`、`song`、`audio`、`radio`、`sound`、`podcast`、`artist`、`album`、`dj`、`mp3`、`bandcamp`、`soundcloud`、`audiomack`、`audius`、`audioboom`、`hypem`、`jiosaavn`、`gaana`、`hungama`、`zing`、`palco`、`tunein`、`mixcloud`、`yandexmusic`、`qq音乐/网易云/酷我/酷狗/咪咕/电台/音乐`（中文 desc）等；另加**站点名白名单**（`Bandcamp`、`Audius`、`Hypem`、`LastFM`、`Idagio`、`Jamendo`、`Freesound`、`EpidemicSound` 等）。
- **强负信号**（一票否决）：`video`、`tv`、`news`、`movie`、`film`、`sport`、`game`、`porn`、`adult`、`anime`、`course`、`lecture`、`webinar`、`trailer`。
- **评分**：`score = 正信号计数（加权）− 负信号计数×2`；`score ≥ 阈值` → `category ∈ {music, radio, podcast}` 进候选；`0 < score < 阈值` → 进 `review_required` 边界清单；否则丢弃。
- 类别细分：`radio`/`podcast` 关键词优先归类（这两类默认**不自动接入**，因为"音乐可播放"目标下电台/播客多数是 HLS，见 §1.1 约束 2——但保留在清单里让人决定）。

**实测数据**（本 agent 用 supportedsites.md 复现过滤逻辑）：1731 条 extractor 中强信号过滤得 **87 个 base 站点**，含少量噪音（`imgur`、`tiktok`、`vimeo`、`wordpress`、`smotrim` 等，因 desc 含 "track/audio" 误报）——正是 `review_required` 的用武之地。真实音乐来源规模估计 **60–70 个**（含电台/播客），远小于全库，人工审核成本可接受。

### 2.3 输出格式（`sync/out/candidates.json`）

```json
{
  "schema_version": 1,
  "generated_at": "2026-08-06T10:00:00+08:00",
  "yt_dlp": { "revision": "2026.08.06", "commit": "fdcc954df" },
  "counts": { "extractors_total": 1838, "candidates": 87, "review_required": 23, "new_since_last": 2, "changed_since_last": 5 },
  "sources": [
    {
      "ie_key": "SoundcloudSearchIE",
      "ie_name": "soundcloud:search",
      "site": "soundcloud",
      "desc": "Soundcloud search; \"scsearch:\" prefix",
      "netrc": "soundcloud",
      "working": true,
      "broken": false,
      "search": { "search_key": "scsearch" },
      "category": "music",
      "music_score": 0.92,
      "match_reasons": ["name:sound", "desc:search"],
      "status": "new",
      "already_supported": false
    }
  ],
  "review_required": [
    {
      "ie_key": "ImgurIE",
      "ie_name": "imgur",
      "category": "unknown",
      "music_score": 0.3,
      "match_reasons": ["desc:audio"],
      "note": "含 audio 但属图床，建议人工确认后移入 ignore 名单"
    }
  ]
}
```

字段说明：
- `status`：与上次清单 diff 的结果，取值 `new` / `changed`（上游改动） / `broken`（`_WORKING=False` 或 desc 含 "Currently broken"） / `removed` / `unchanged`。
- `already_supported`：对照 `sync/sources.json` 注册表（§3.3），供报告过滤"已接入"项。
- 一个 `site` 下多个 `ie_key`（如 `soundcloud` 8 个 IE）在生成器内聚合成**一个接入候选**，因为接入粒度是"来源"，不是"extractor 类"。

### 2.4 生成器脚本与运行

```
tools/sync/
├── scan_ytdlp.py        # 枚举 + 评分 + 输出 candidates.json + change-report.md
├── gen_adapter.py       # (中期) 读 sources.json 生成/更新 Rust 接入代码，见 §3.4
├── sources.json         # 人工维护注册表：site -> {prefix, rust_name, category, status}
└── out/                 # 生成产物（gitignore）
```

- `scan_ytdlp.py` 运行方式：`uv run --python 3.12 tools/sync/scan_ytdlp.py --repo ../yt-dlp --registry tools/sync/sources.json`（yt-dlp 要求 Python ≥ 3.10；本地 3.9.6 不可直接跑，用 uv/venv 解决）。`PYTHONPATH` 指向 `../yt-dlp`。
- **构建期零依赖**：生成器是 dev-time 工具，`../yt-dlp` 只是资源仓库，绝不进 `Cargo.toml`/tauri bundle（与 deleted spec 的"reference + oracle"定位一致）。
- diff 机制：`git -C ../yt-dlp fetch` → 用 `git log HEAD..@{u} --oneline -- yt_dlp/extractor/` 拿到变更面，但**候选清单本身以"重跑枚举器 + JSON 对比"为准**（枚举 API 才是真源，git log 只用于报告上下文）。

---

## 3. 五步接入的自动化可行性表

### 3.1 总表

| 步骤（extension-guide） | 自动化 | 可行性 | 工具/机制 | 人工残留 |
|---|---|---|---|---|
| 1. `extractor/<source>/` 协议代码 | **低** | 不可自动 | 生成**模块骨架**（`mod.rs` + `search.rs`/`player.rs` 桩，返回 `PlaybackError::Unimplemented`） | 协议逆向、流选择、headers、加密/signature —— **核心人工，无法回避**；可借助 yt-dlp 对应 extractor 源码 + 实际 `-J` 输出做逆向参考与 fixture |
| 2. `playback/<source>.rs` adapter | **高**（~90%） | 模板生成 | extension-guide §1 已有完整骨架；唯一变量 = `SourceRef::Xxx { <field> }` 字段名与 `Xxx` 类型名（来自 sources.json）；`Track → PlayableEntry` 映射是机械的（`track_to_view` 已是共享函数） | 确认业务语义（如 artists 拼接、空 album） |
| 3. `SourceKind`/`TrackId`/`SourceRef` 标识层 | **高**（100%） | 机械扩展 | 在 `model.rs`/`types.rs` 的 **marker 块**内重写（§3.2）；`TrackId::FromStr`/`Display`、`SourceRef::kind()`、`TrackMeta::from/to_source_ref` 全部模板化 | 前缀命名决策（一次性、不可变，见 §3.3） |
| 4. `runtime.rs` 注册 | **高**（100%） | 机械插入 | marker 块内 `use` + 两行 `register(...)`；`resolver-only` vs `search` 由 sources.json 的 `has_search` 决定 | 无 |
| 5. `track_to_entry` 前缀表 | **高**（100%） | 机械插入 | `search.rs` marker 块内加 match arm（前缀来自 sources.json）；`Wechat` 式"搜索跳过"来源记为 `search: false` | 无 |
| 6. 测试骨架 | **中** | 模板化 | contract 测试（ID 前缀 round-trip、resolver 空流错误路径）可生成；smoke（真实播放）**不可自动** | smoke 测试人工跑一次或留 CI 夜间任务 |
| 7. DB 兼容 | **高** | 零新代码 | `MetaValue::Extractor(String)` 已是通用载体，只加 `strip_prefix` 分支 | **前缀不可变纪律**（改前缀 = 丢收藏/最近播放） |

### 3.2 marker 块机制（关键工程决策）

代码生成必须**幂等**，否则重跑会重复插入。推荐在 Rust 文件里声明生成区：

```rust
// ==== sync-generated:begin source_kinds ====
// ==== sync-generated:end source_kinds ====
```

`gen_adapter.py` 每次**整体重写 marker 块内容**（块内不做增量 diff），然后 `cargo fmt` + `cargo check` 收尾。块外任何人工改动不受影响。比 `ast_edit`（一次性 codemod，重跑会重复匹配）更适合"持续同步"场景；`ast_edit` 留作一次性迁移工具。

### 3.3 `sync/sources.json` 注册表（人工决策落点）

```json
{
  "soundcloud": { "prefix": "sc", "rust_name": "Soundcloud", "has_search": true,  "category": "music", "adopted_at": "2026-08-06", "status": "in_progress" },
  "bandcamp":   { "prefix": "bc", "rust_name": "Bandcamp",   "has_search": false, "category": "music", "adopted_at": null, "status": "candidate" }
}
```

- 人工只在这里填 **3 个决策**：`prefix`（≤8 字符小写，与 `yt:`/`bili:` 及既有项查重）、`rust_name`、`has_search`。
- `prefix` 一旦用于生成并发布，**永不可变**（DB 里存 `"<prefix>:<id>"`）。
- 生成器以它为准产出 `already_supported` 与变更报告。

### 3.4 `gen_adapter.py` 一键生成流程

输入：`sources.json` 中 `status == "in_progress"` 且带 `prefix/rust_name` 的条目 + `tools/sync/templates/adapter.rs.tmpl`。
输出（全部落盘后 `cargo fmt`）：

1. `extractor/<site>/` 骨架（mod.rs + 桩实现）；
2. `playback/<site>.rs`（完整 adapter，extractor 桩返回后即可编译）；
3. `model.rs`/`types.rs`/`search.rs`/`runtime.rs` 的 marker 块重写；
4. `playback/<site>.rs` 内嵌 contract 测试 + `public.rs` 不需要动（注册表驱动，extension-guide §3 明确"不要新增 command"）。

人工剩余工作量 = **写 extractor 协议代码** + **跑一次 smoke**。接入成本从"跨 7 个文件、2–3 天"降到"一个文件 + 一晚上"（逆向本身的时间另算）。

---

## 4. 可行性边界（诚实评估）

yt-dlp 一个站点 extractor 是几百行 Python（qqmusic 492、bandcamp 544、youtube/_base 1349），依赖其庞大 util 工具箱（`traverse_obj`、签名算法、impersonation、cookies 体系）。**自动翻译成 Rust 不可行**，唯一可行的是：

| 方案 | 是什么 | 成本 | 收益 | 结论 |
|---|---|---|---|---|
| **a. 自动发现 + 注册骨架 + TODO 标记** | 本提案主线：生成器出候选 → 人决定做不做 → 生成器铺好 2–6 步 → 人只写 extractor | 生成器 2–3 天 + 每次接入 1 个 extractor 的逆向 | 接入成本降一个量级，零运行时风险 | **推荐（短期+中期）** |
| **b. 只生成候选清单 + 变更报告** | 纯工具 + 文档，人做全部接入 | 半天 | 发现能力，无自动化 | **先行阶段**（本提案短期即含） |
| **c. 通用代理（yt-dlp 子进程）** | 见 §5 | 高（打包/性能/架构） | 新来源零 Rust 代码 | **不推荐为主线**，仅限 oracle + URL 兜底 |

---

## 5. 通用代理方案深入评估（方案 c）

### 5.1 架构草图

```text
Tauri app
  └── YtdlpProxyAdapter (SearchProvider + PlaybackResolver)
        ├── 搜索: spawn `yt-dlp -J --flat-playlist "scsearch10:<kw>"` → 解析 entries[]
        └── 播放: spawn `yt-dlp -J --no-playlist -f bestaudio/best <url>` → 过滤 formats[]
```

分发形态：Tauri 2 的 `bundle.externalBin` + `tauri::process::Command::new_sidecar`（把 yt-dlp 官方 PyInstaller 预编译二进制 `yt-dlp_linux`/`yt-dlp_macos`/`yt-dlp.exe` 打成 sidecar，`resources/` 目录，随包分发）。官方二进制含 curl_cffi（YouTube 等站点的 impersonation 必需）。

### 5.2 代价逐项分析

**① 打包（Python 运行时）**
- yt-dlp 纯 Python（requires-python ≥ 3.10，核心零硬依赖），**无需自带解释器**：上游发布 PyInstaller 预编译二进制（30–50MB/平台），`externalBin` 直接收编。省掉自建 pyinstaller 流水线。
- 代价真实存在：app 体积 +30–50MB（当前 bundle 只有 deb/rpm）；macOS 上 sidecar 未签名 → Gatekeeper 拦截，需要公证/签名流程；yt-dlp 每周发版，跟随更新的供应链成本。
- **移动端不可行**（yt-dlp 无 Android/iOS 侧车分发先例，iOS 禁子进程外部解释器）——而 deleted spec 的目标平台明确含 Android/iOS，这是**架构性否决**。

**② 启动与性能**
- PyInstaller onefile 每次调用解包 ~30MB 到临时目录（0.5–2s）+ Python 启动 + yt_dlp import（~0.3s）+ 网络提取（1–5s）→ 单查询 **2–8s**。
- **搜索是串行枚举**（§1.1 发现 1）："all" 搜索 = N 站点 × 2–8s 串行。两个自研来源下已可接受，代理化 10 个站点后 UI 必然超时。缓解：持久化 worker 进程（Python 侧起 YoutubeDL 长驻 + stdin/stdout JSON-RPC，省掉解包/import 开销，单查询压到网络时间）——但这本身就是一个中等工程（进程生命周期、取消、崩溃恢复、协议版本化）。
- 取消语义：`PlaybackService` 取消需 kill 子进程（或向 worker 发 cancel），现有 `ExtractorContext` 的取消钩子要桥接。

**③ JSON 解析层**
- `-J` 输出是 best-effort 的 info dict，字段不保证存在：Rust 侧要写 `info_dict → Track / PlaybackManifest` 映射（`title/artist(s)/uploader/duration/thumbnails/formats`），需要 fixture 回归测试锁行为；yt-dlp 周更，字段语义漂移需要持续跟进——**这正是"维护成本转移"：从写 extractor 变成维护映射层 + 版本锁定**。
- **格式过滤（播放约束）**：§1.1 发现 2 —— symphonia 只吃单文件音频。`-f bestaudio/best` 可能选到 HLS/DASH；Rust 侧必须过滤 `formats[]`（`protocol ∈ {http,https}`、`vcodec == none`、`ext ∈ {mp3,m4a,aac,opus,ogg,flac,wav}`），失败要递归回退。**只提供 HLS-AAC 的站点（大量电台/播客）在代理方案下不可播**，除非另写 HLS 拉流组件（新的运行时依赖，超出"零代码"承诺）。

**④ 架构融合（与现有体系的冲突点）**
- `SearchProvider::search(&ExtractorContext)` 契约是"共享 context 的 HTTP 客户端"，代理适配器绕开它起子进程——边界仍成立（adapter 内封装），但 `ExtractorContext` 的取消/日志/代理配置全被旁路。
- `SourceKind` 粒度决策：逐站点变体（`Soundcloud/Bandcamp/...`，与现状一致，靠 §3 代码生成器补齐，**仍然需要每次生成+重编译**，只是不需要逆向）vs 单一 `Ytdlp{site}` 变体（零代码生成但 `TrackId` 前缀变成 `ydlp:<site>:<id>`，来源身份混在一起，`kind()`/catalog/去重语义退化）。前者才是干净答案——所以"零 Rust 代码"实际上打折成"零逆向代码，代码生成器照跑"。
- **搜索缺口**：全库 `_SEARCH_KEY` 只有 10 个，音乐站点仅 soundcloud（§1.3）。Bandcamp/QQ音乐/网易云等 yt-dlp 自己都搜不了 → 代理方案的新来源**大多只能做 resolver-only**（URL 粘贴/收藏恢复场景）。而当前 `ParseUrl` 只验证微信链接（`ParseUrl.tsx` `validateWechatUrl`），泛化 URL 解析是另一个前端+command 工程。
- 质量特性：yt-dlp 不保证某站点"音乐可播放"（可能返回视频格式、直播、需登录）；`--ignore-no-formats-error` 等失败路径要逐条映射到 `PlaybackError`。

### 5.3 结论：推荐 or 不推荐

**不推荐作为默认主路线**。核心理由按权重排序：

1. **架构先例否决**：deleted `docs/spec-yt-dlp.md` 明确"不直接依赖 yt-dlp binary"（跨平台目标），代理方案与之一致性冲突；
2. **播放约束**：HLS/DASH 站点不可播，等于把"自动获得支持"限制在直链音频站点子集，还要自写 format 过滤；
3. **搜索缺口 + 串行枚举**：覆盖面和延迟双重不达标，代理解决不了"搜索"这个核心入口；
4. **打包/供应链**：+30–50MB、macOS 签名、周更跟随，成本落在最不可控的地方；
5. 收益（长尾站点零逆向）被上述成本吃掉大半。

**有条件保留的两个用途**（强烈建议，但不叫"通用代理主线"）：

- **开发期 oracle**：复活 deleted spec §8 —— 用 `yt-dlp -J` 对固定 query/URL 生成 oracle JSON，与自研 extractor 差分（元数据/format/headers），上游变更时自动告警。`music-cli`（`cli/`）已直连 `ExtractorContext`，接 fixture 成本低。这是"上游更新自动同步"最诚实、收益最高的落点。
- **生产期 URL 兜底 resolver**：若产品接受"粘贴链接解析任意站点"，把 `ParseUrl` 泛化，加一个 `YtdlpFallbackResolver`（resolver-only，不进 `SearchRegistry`，用户主动粘贴才付子进程成本）。可作为**独立 feature 决策**，不阻塞本提案主线。

---

## 6. 维护闭环：同步 → 任务 → 结案

### 6.1 同步工作流（手动或 CI 可选）

```text
git -C ../yt-dlp fetch
→ tools/sync/scan_ytdlp.py --repo ../yt-dlp --registry tools/sync/sources.json
→ 输出 sync/out/candidates.json（新状态）+ change-report.md（与上次 diff）
→ 人工审 report：勾选采纳/忽略
```

`change-report.md` 四段：

1. **🆕 新候选来源**（`status: new`，进 candidates.json）—— 决定做不做；
2. **⚠️ 已接入来源的上游变更**（`status: changed` 且 `already_supported: true`）—— **这是最常发生的维护任务**：已接入站点上游改了协议，需要人工核对自研 extractor（oracle 差分直接给出差异点）；
3. **💔 上游标记损坏**（`_WORKING=False` / desc 含 "Currently broken"）；
4. **🗑️ 上游移除**。

### 6.2 新来源 → GitHub issue（模板化）

`.github/ISSUE_TEMPLATE/new-source.md`（参照上游 `devscripts/make_issue_template.py` 的思路，但我们的模板是**接入 checklist** 而非 bug 报告）：

```markdown
---
title: "🎵 新来源候选：{{site}}（{{ie_name}}）"
labels: ["source", "new-source"]
---

- [ ] 审核：确认是"音乐可播放"来源（直链音频优先；HLS/DASH 标记为受限）
- [ ] 决策：`sync/sources.json` 填入 `prefix` / `rust_name` / `has_search`
- [ ] 运行 `tools/sync/gen_adapter.py --site {{site}}`（自动完成步骤 2–5 + 测试骨架）
- [ ] 人工实现 `extractor/{{site}}/`（参考 `../yt-dlp/yt_dlp/extractor/{{file}}.py`）
- [ ] contract 测试通过 + 一次真实播放 smoke
- [ ] 更新 `docs/CHANGELOG-sources.md`
```

- 生成器可**自动预填** body（从 candidates.json 拿 `ie_name/desc/netrc/working/search_key`），人只点"创建 issue"。
- 变更/损坏报告走同一模板体系（`label: source-sync`），由维护者决定是否开 issue。

### 6.3 结案与 changelog

- `docs/CHANGELOG-sources.md`：每接入一个来源一行（site、prefix、接入日期、yt-dlp commit），生成器在 `adopted_at` 写入时 append 建议行。
- `sync/sources.json` 的 `status` 流转：`candidate → in_progress → adopted / rejected / ignored`（rejected 记录原因，避免重复评估）。

---

## 7. 推荐路线（短期/中期/长期）

| 阶段 | 交付物 | 工作量 | 依赖 |
|---|---|---|---|
| **短期（先行，零代码风险）** | ① `tools/sync/scan_ytdlp.py` + `sources.json` 骨架 + 输出格式；② 首份 `candidates.json`（实测 ~87 站点）+ 人工审核一遍；③ issue 模板 + `CHANGELOG-sources.md`；④ 同步流程写进 README 或 CI 手动 job | 2–3 天 | 仅 dev-time，构建/运行零影响 |
| **中期（把人工压到最小）** | ⑤ marker 块机制 + `gen_adapter.py` + adapter 模板；⑥ 以 soundcloud 或 bandcamp 为试点跑通"一键生成 + 人工 extractor"；⑦ 复活 oracle 差分（`music-cli` + fixture + 固定 query 集），上游变更自动告警 | 1–2 周 | 短期产物 |
| **长期（决策点，不默认做）** | ⑧ 若产品诉求=长尾站点且桌面优先：`YtdlpFallbackResolver`（URL 兜底，resolver-only）+ `ParseUrl` 泛化 + `externalBin` sidecar 打包（含 macOS 签名评估）；移动端诉求出现则否决 | 2 周+ | 产品决策 |

**判断准则**：每接一个来源若需要 >1 个 extractor 逆向工作日，或出现第 3 个"纯 HLS 站点"诉求，触发 ⑧ 的重新评估——用数据说话，不预设立场。

---

## 8. 附：证据索引（本次调研读取位置）

- music_demo：`docs/extension-guide.md`（全）、`docs/architecture.md`（全）、`docs/contracts.md`（command 表）、`src-tauri/src/playback/model.rs`、`search.rs`、`runtime.rs`、`public.rs:29-60`（串行枚举）、`src-tauri/src/types.rs`、`src-tauri/src/extractor/model.rs`、`src-tauri/tauri.conf.json`（bundle targets）、`src/components/ParseUrl.tsx`（仅微信验证）、`.github/workflows/`（build/release）、`git show HEAD:docs/spec-yt-dlp.md`（已删先例：不依赖 yt-dlp binary + oracle 工作流）。
- yt-dlp：`supportedsites.md`（1731 条目抽样）、`devscripts/make_supportedsites.py`（枚举器全貌）、`yt_dlp/extractor/__init__.py`（`gen_extractor_classes/list_extractor_classes`）、`yt_dlp/extractor/common.py`（`IE_NAME/_VALID_URL/IE_DESC/_WORKING/_NETRC_MACHINE/description()`/`SearchInfoExtractor._SEARCH_KEY`）、`yt_dlp/extractor/soundcloud.py`（`SoundcloudSearchIE`、`_get_n_results→playlist_result`）、`yt_dlp/extractor/_extractors.py`（941 文件 import）、`yt_dlp/extractor/*` 行数统计（qqmusic 492 / bandcamp 544 / kuwo 352 / youtube/_base 1349）、`_SEARCH_KEY` 全集 10 个、`pyproject.toml`（requires-python ≥3.10、核心零依赖）、`yt_dlp/plugins.py`（插件机制，未采纳但可参考）。
