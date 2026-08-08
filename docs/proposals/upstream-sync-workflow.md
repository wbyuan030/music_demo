# 提案：yt-dlp 上游更新 → music_demo 自动同步（工作流层）

> ⚠️ **已废弃（2026-08-06）**：本提案的核心机制（关键词/路径/规则过滤、变更报告 + Draft PR、人工实施）被用户否决，由 `docs/proposals/agent-sync-workflow.md`（全量同步 + agent 翻译 + 测试驱动 PR）取代。保留仅供历史参考。

> 范围：监测 / 变更分类 / 产物形态 / 影响评估 / 失败降级 / 新来源发现。**不含**协议字段映射（另见字段映射提案）。
> 原则声明：music_demo 的 extractor 是自研 Rust，**"同步"永远不意味着复制 yt-dlp 代码**，而是"结构化、可行动的变更情报 + 人工实施"。本提案的一切自动化都服务于这一事实，不承诺自动翻译 Python → Rust。

---

## 0. 结论摘要

- **监测**：GitHub Actions 定时任务（每日 1 次）+ 手动 `workflow_dispatch`；本地 CLI 复用 `../yt-dlp` clone（`git log <last_sha>..HEAD`），天然兼容"本机 clone 已手动拉新"。Release/RSS 只作版本漂移心跳，不作触发源。
- **分类**：文件路径分级（Tier A/B/C）+ commit subject 前缀 + 关键词三重过滤，输出每 commit 的 RELEVANT/WATCH/IGNORE 决策与理由。
- **产物**：自动生成 **变更报告 markdown + Draft PR（`upstream-sync` label）**；报告 PR 合并 = 人工确认已阅；实际 Rust 改动走独立代码 PR 过现有 `build.yml` 门禁。**永不自动合并**。
- **影响评估**：`scripts/upstream-sync/watchlist.yaml` 记录我们依赖的协议符号（本文 §5 给出从代码里实读的具体清单），diff 命中即升严重级。
- **新来源发现**：diff `supportedsites.md` + 新增 extractor 文件，按音乐站名单过滤，产出"候选新来源"清单（仅情报，实施仍是人工）。
- **降级**：状态锁 `.sync/upstream-state.json`（bot commit 维护，yt-dlp 自身 version.py 同款模式）；网络失败只挂 sync job 并开 issue 告警，**绝不阻塞**主 CI；上游 force-push/路径重构有专门兜底。

**人工 / 自动总表**（详见表末 §10）：

| 环节 | 自动化 | 人工 |
|---|---|---|
| 上游监测（fetch/API/版本漂移） | ✅ 全自动 | — |
| 变更分类与过滤 | ✅ 全自动（规则可维护） | 规则维护（tiers/watchlist） |
| 变更报告生成 | ✅ 全自动 | — |
| Draft PR 开出/更新 | ✅ 全自动 | — |
| 报告审阅（合并报告 PR） | — | ✅ 必须 |
| 协议字段映射 + Rust 实施 | — | ✅ 必须 |
| 代码同步 PR 合并 | CI 门禁自动跑，合并人工 | ✅ 必须 |
| 新来源候选发现 | ✅ 全自动 | 决定是否立项 |

---

## 1. 调研事实基础（提案依据，全部实读验证）

| 事实 | 证据 |
|---|---|
| 上游 commit 密度 | master 近 8 周 8/12/20/30/6/1/7 条/周；近 120 天共 134 条。**绝大多数与我们无关**（941 个 extractor 文件） |
| 上游发版节奏 | 2026 年 tag：01.29、01.31、02.04、02.21、03.03、03.13、03.17、06.09、07.04；2024 年 22 个、2025 年 26 个 → **平均每月 1–2 个 release，且间隔不均（3 月→6 月断档）** |
| 版本机制 | `yt_dlp/version.py` 由 `devscripts/update-version.py` 自动生成：`__version__` + `RELEASE_GIT_HEAD` + `CHANNEL` + `ORIGIN`。**一个 release 同时可定位版本串与 git SHA** |
| 上游 CI 模式 | 自己的 `core.yml` 就把 `yt_dlp/extractor/youtube/**` 当核心路径跑；发版是 `workflow_dispatch` 手动触发 + `release-master.yml` 按路径过滤自动发 prerelease。**路径过滤是 yt-dlp 官方验证过的模式** |
| youtube extractor 布局 | 已拆包：`yt_dlp/extractor/youtube/`（`__init__.py` 汇总导出；`_base.py` 客户端配置；`_video.py`=YoutubeIE；`_search.py`/`_tab.py`；`jsc/` JS 挑战求解器；`pot/` Proof-of-Origin-Token 提供器）。单文件 `youtube.py` 已于 2023 拆包移除 |
| bilibili extractor | 单文件 `yt_dlp/extractor/bilibili.py`；近 60 天仅 1 条相关 commit（`[ie/bilibili] Fix API extraction`） |
| supportedsites.md | 由 `devscripts/make_supportedsites.py` 从 extractor 类生成并**提交进 repo**（`Makefile:143`）→ 可直接 `git diff` 发现新站点 |
| 我方依赖（实读代码） | 见 §5 watchlist；核心：WEB_REMIX `1.20260707.12.00`（YT Music 搜索）、ANDROID_VR `1.65.10`（player）、硬编码 `INNERTUBE_API_KEY`、youtubei/v1 两个端点、流式字段、Bilibili WBI 签名全家桶 |
| 我方 CI 现状 | `.github/workflows/build.yml`（push/PR 到 main|master → cargo test + tsc + tauri build）、`release.yml`（tag v*）。**无定时任务、无依赖更新自动化**；当前分支 `refactor/extractor` 不在 CI 触发分支上 |
| 我方现状 gap | 全仓库无任何 yt-dlp 版本追踪/锁文件（仅 2 处注释提及） |

**一个必须正视的上游事实**：yt-dlp 已不再硬编码 InnerTube API key，而是运行时从页面 `ytcfg` 动态取（`yt_dlp/downloader/youtube_live_chat.py:155` `try_get(ytcfg, lambda x: x['INNERTUBE_API_KEY'])`）；我方仍硬编码 `AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8`。这本身就是"上游演进 → 我方脆弱点"的活例子，应进 watchlist。

---

## 2. 推荐架构总览

```mermaid
flowchart LR
    subgraph Up["上游 yt-dlp"]
        GH[github.com/yt-dlp/yt-dlp]
        CLONE[本地 clone ../yt-dlp]
    end
    subgraph M1["① 监测（全自动）"]
        CRON[cron 每日 + workflow_dispatch<br/>+ 本地 CLI 手动]
        API[GitHub API: commits/tags/releases]
        LOG[git log last_sha..origin/HEAD]
    end
    subgraph M2["② 分类过滤（全自动）"]
        TIERS[tiers.yaml 路径分级]
        WATCH[watchlist.yaml 关键词]
        SUBJ[commit subject 前缀]
    end
    subgraph M3["③ 报告（全自动）"]
        REP[upstream-report-&lt;date&gt;.md]
        ST[.sync/upstream-state.json]
        NEWSRC[新来源候选]
    end
    subgraph M4["④ 门禁（人工）"]
        DPR[Draft PR label=upstream-sync]
        LOCAL[本地 CLI 终端输出]
        H[开发者 review + Rust 实施]
    end
    subgraph M5["⑤ 落地"]
        CPR[代码同步 PR]
        CI[build.yml 既有门禁]
    end

    GH --> CLONE
    CRON --> API
    API --> LOG
    CLONE --> LOG
    LOG --> M2
    TIERS --> M2
    WATCH --> M2
    SUBJ --> M2
    M2 --> REP
    M2 --> NEWSRC
    REP --> DPR
    REP --> LOCAL
    ST --> REP
    DPR --> H
    LOCAL --> H
    H --> CPR
    CPR --> CI
```

**核心设计决策**：
1. **一个脚本、两个入口**：`scripts/upstream-sync/sync.py` 是唯一逻辑实现，CI 与本地 CLI 共用 → 行为一致、测试一次。
2. **状态锁在 git 里**（bot commit 维护 `.sync/upstream-state.json`）：CI 与本地天然一致，历史可审计，回滚即 revert——与 yt-dlp 的 `version.py` 自动生成提交是同一模式。
3. **上游只读、永不执行**：管道只读 git 对象与 diff，**禁止** `pip install yt-dlp` 或运行其代码（缩小供应链面）。
4. **报告 PR 与代码 PR 分离**：报告 PR 合并 = "已阅确认 + 状态推进"；代码 PR 过既有 `build.yml`。自动化的终点是情报与人，不是 merge 按钮。

---

## 3. 模块一：上游监测

### 3.1 候选方案对比

| 方案 | 优点 | 缺点 | 结论 |
|---|---|---|---|
| A. 定时 Actions + GitHub API | 无本地依赖、可审计、天然有 token | 需要调度开销 | ✅ 主通道 |
| B. 订阅 GitHub Releases RSS | 零成本 | release 频率低且不均（月 1–2 次）；YouTube 修复往往在 release 之间先上 master；RSS 只给 release 不给 commit | ❌ 仅作版本漂移心跳 |
| C. 纯本地 clone + git log | 复用现有 `../yt-dlp`；无网络也能跑 | 机器下线即失联 | ✅ 本地通道（与 A 并存） |
| D. 每次 PR/CI 都顺带检查 | 零额外调度 | 我们 PR 频率低，漂移检测延迟大 | ❌ |

### 3.2 推荐机制

**CI 通道**（`upstream-sync.yml`）：

```yaml
name: Upstream sync check
on:
  schedule:
    - cron: "17 3 * * *"        # 每日 03:17 UTC（避开整点洪峰）
  workflow_dispatch: {}          # 手动触发
concurrency:
  group: upstream-sync
  cancel-in-progress: true       # 防叠加
permissions:
  contents: write                # 仅本 workflow；bot commit 用
jobs:
  sync:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with: { fetch-depth: 0 }   # 需要历史判断 last_sha 可达性
      - run: pip install PyYAML requests
      - run: python scripts/upstream-sync/sync.py --ci
      - run: python scripts/upstream-sync/sync.py --open-pr   # 仅在发现 RELEVANT 时
```

- **触发周期建议：每日 1 次**。依据：master 周 commit 数 8–30，YouTube 相关可行动改动约 2–6 条/月；我方响应端是人工，日级足够，想更快手动 dispatch。
- 探测内容（脚本内，优先级从低到高）：
  1. `git ls-remote origin master` → 得最新 SHA，与状态锁比对；
  2. 若有新 SHA：`git clone --depth 1 --branch master` 到临时目录（或 `git fetch` 现有 mirror，见 §8 缓存策略），产出 `git log --format=%H%x09%ad%x09%s last_sha..HEAD`；
  3. `git ls-remote --tags origin` + GitHub API `releases/latest` → 记录最新版本串，与状态锁里的 `upstream_version` 对比出漂移天数。

**本地通道**（`scripts/upstream-sync/sync.py` 无参运行，纯 git，零 API）：

```
# 兼容"本机 clone 已手动拉新"的关键：
git -C ../yt-dlp fetch origin        # 可选 --offline 时跳过
NEW=$(git -C ../yt-dlp log --oneline "$LAST_SHA..origin/master")
```

- 若 `origin/master` 与本地 `master` 一致或更旧 → 直接用本地 master（覆盖手动 fetch/pull 场景）；
- 若断网 → `--offline`：只读 clone 现有状态，不 fetch；
- 版本串从 `../yt-dlp/yt_dlp/version.py` 正则提取（`__version__|RELEASE_GIT_HEAD|CHANNEL`）。

**不采用 RSS 的理由（证据）**：2026 年 tag 分布 01.29→07.04 共 10 个，3 月中旬到 6 月 9 日断档近 3 个月——release 触发会漏掉断档期内 master 上所有 YouTube 修复（如 07.09–07.23 的 5 条 youtube client 维护 commit 都在 07.04 release 之后）。commit 粒度必须从 git 拿。

---

## 4. 模块二：变更分类与过滤

### 4.1 路径分级（`tiers.yaml`，核心规则，用 yt-dlp 官方路径过滤模式同款思路）

```yaml
tiers:
  tier_a:            # RELEVANT：直接影响我方依赖
    - "yt_dlp/extractor/youtube/**"      # 主战场：客户端配置、player/search 解析、jsc/pot
    - "yt_dlp/extractor/bilibili.py"     # 我方第二来源（单文件）
  tier_b:            # WATCH：机制级变化可能波及所有 extractor
    - "yt_dlp/extractor/common.py"       # 基类能力（_extract_formats 等）
    - "yt_dlp/extractor/__init__.py"
    - "yt_dlp/extractor/extractors.py"   # lazy 注册表（新来源发现也看它）
    - "yt_dlp/jsinterp.py"               # JS 解释器：YouTube 反混淆风向标
    - "yt_dlp/aes.py"
    - "yt_dlp/networking/**"
    - "yt_dlp/utils/**"                  # 仅命中 watchlist 关键词时升 Tier A
  ignore:            # IGNORE：与我们无关
    - "yt_dlp/extractor/*.py"            # 其他 900+ 站点（注意不含 youtube/ 子目录）
    - "yt_dlp/extractor/*/"              # 其他站点包
    - "yt_dlp/postprocessor/**"
    - "yt_dlp/downloader/**"
    - "test/**" "devscripts/**" "docs/**" ".github/**" "README.md" "Changelog.md"
    - "bundle/**" "pyproject.toml" "Makefile"
```

理由（基于实读）：
- `youtube/**` 含 `_base.py`（`_INNERTUBE_CONTEXTS` 客户端配置、`GVS_PO_TOKEN_POLICY`）、`_video.py`（streamingData 解析/格式提取）、`jsc/`+`pot/`（YouTube 上线的 JS 挑战 / PO token 对策）——**YouTube 反制手段的演进全部落在这里**；
- `common.py` 的格式提取/解密辅助是我们的"间接依赖"（我们自研但概念同源），仅关键词命中时报警；
- 941 个 extractor 文件里其余站点改动一律 IGNORE，省 95% 噪音。

### 4.2 commit subject 前缀规则（次要信号，增强而非主判据）

yt-dlp 的 subject 格式高度规整（`[ie/youtube] ...`、`[core] ...`、`[misc] ...`、`[utils] ...`）。正则：

```
RELEVANT: ^\[(ie/youtube|ie/bilibili|ie/youtube:\S+)\]
WATCH:    ^\[(core|utils|jsinterp|networking)\]
IGNORE:   其余（含 [ie/<其他站>]、[fd/*]、[pp/*]、[misc]、[cleanup]）
```

**以文件路径为 ground truth，subject 只作报告里的展示与二次校验**（subject 可能改名/缺失，路径不会）。

### 4.3 决策矩阵

对每个新 commit：

| 条件 | 分类 | 动作 |
|---|---|---|
| 命中 Tier A 路径 | RELEVANT | 进报告主体 + 触发 Draft PR |
| 命中 Tier B 路径且命中 watchlist 关键词（§5） | RELEVANT | 同上 |
| 命中 Tier B 但无关键词 | WATCH | 报告附录一行，不开 PR |
| Tier B 命中关键词但路径在 ignore | RELEVANT（警告） | 报告标注"规则异常"，提示维护 tiers |
| 其余 | IGNORE | 只进统计计数 |

产出：`commits.json`（每 commit：sha/subject/日期/文件列表/分类/命中关键词/严重级）+ 人类可读报告。

---

## 5. 模块三：变更影响评估（watchlist）

### 5.1 watchlist.yaml（依赖符号清单，全部实读自我方代码）

```yaml
# 命中任意 token 即升严重级并给出建议动作；action 文案由脚本拼接
watchlist:
  youtube:
    - name: innertube_api_key
      tokens: ["INNERTUBE_API_KEY", "AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8", "innertubeApiKey"]
      note: 我方 api.rs 硬编码该 key；yt-dlp 已改动态取 ytcfg。key 更换=我方必挂
      action: "核对 api.rs::INNERTUBE_API_KEY"
    - name: web_remix_client
      tokens: ["WEB_REMIX", "'1.20260707.12.00'", "clientVersion"]
      note: 我方搜索用 WEB_REMIX 1.20260707.12.00（api.rs:96-97）；yt-dlp _base.py 同款
      action: "对齐 client_version（搜索链路）"
    - name: android_vr_client
      tokens: ["ANDROID_VR", "'1.65.10'"]
      note: 我方 player 用 ANDROID_VR 1.65.10（api.rs:154-155）；yt-dlp 注释：>1.65 可能只回 SABR 流
      action: "核对 client_version（播放链路）"
    - name: innertube_endpoints
      tokens: ["youtubei/v1/player", "youtubei/v1/search", "music.youtube.com", "INNERTUBE_HOST", "prettyPrint"]
      note: 我方两个端点（api.rs:109,167）
      action: "核对端点/主机"
    - name: streaming_data
      tokens: ["streamingData", "adaptiveFormats", "signatureCipher", "cipher", "expiresInSeconds", "videoDetails"]
      note: 我方 types.rs/player.rs 解析字段；yt-dlp 改格式提取逻辑=响应形状变化的强信号
      action: "核对响应字段（types.rs 反序列化）"
    - name: playability
      tokens: ["playabilityStatus", "LOGIN_REQUIRED", "UNPLAYABLE"]
      note: 我方 api.rs::check_playability
      action: "核对播放性判定"
    - name: search_renderers
      tokens: ["MusicShelfRenderer", "MusicResponsiveListItemRenderer", "VideoRenderer", "ContinuationItemRenderer", "continuation", "runs", "navigationEndpoint"]
      note: 我方 types.rs 搜索响应结构
      action: "核对搜索响应结构"
    - name: pot_and_jsc
      tokens: ["po_token", "PO_TOKEN", "GVS_PO_TOKEN_POLICY", "jsc", "nsig", "sig", "player.js"]
      note: 我方未实现 POT/JS 挑战，ciphered 格式直接丢弃（player.rs 注释）。yt-dlp 已观测 2026.07 起 ANDROID_VR 非 HLS 选择性强制 POT（_base.py:229 注释）
      action: "高风险：确认播放是否仍可用；考虑 POT 实现立项"
  bilibili:
    - name: playurl
      tokens: ["x/player/playurl", "fnval", "fnver", "fourk"]
      note: 我方 player.rs:23 参数 fnver=0&fnval=4048&fourk=1
      action: "核对 playurl 参数"
    - name: wbi
      tokens: ["wbi", "w_rid", "mixin_key", "wbi_img", "img_key", "sub_key", "nav"]
      note: 我方 utils.rs WBI 签名全家桶（含 x/web-interface/nav 取 key）
      action: "核对 WBI 签名流程"
    - name: bili_endpoints
      tokens: ["x/web-interface/search/type", "x/web-interface/view", "api.bilibili.com"]
      note: 我方 search.rs/view 端点
      action: "核对端点"
```

### 5.2 匹配机制

对每个 RELEVANT/WATCH commit 的 `git show` diff（`-U3`），token 做子串匹配（不区分大小写）。规则：

- **任一 Tier A commit 命中** → 严重级 `CRITICAL`（进报告头部"必须处理"区）；
- Tier A 未命中但为 youtube/ 改动 → `IMPORTANT`（大概率是客户端轮换或字段微调，仍建议读）；
- Tier B 命中 → `IMPORTANT`；Tier B 未命中 → `INFO`；
- bilibili 单文件改动 → 恒 `IMPORTANT`（低频高价值，60 天仅 1 条）。

报告里每条 commit 输出：`sha | subject | 文件 | 分级 | 命中 token | 建议动作`。

### 5.3 关键现实示例（证明此评估必要性）

实读 `yt_dlp/extractor/youtube/_base.py:229` 附近，ANDROID_VR 客户端上方注释：
> "Since 2026.07, intermittent/selective POT enforcement has been observed for non-HLS formats"，且 `GVS_PO_TOKEN_POLICY` 对 HTTPS/DASH 已标 `required=True`。

我方 `player.rs` 恰恰依赖 ANDROID_VR 直连返回 `adaptiveFormats`、无 POT、无解密——这正是 watchlist 中 `pot_and_jsc` 要抓的信号。**单靠"客户端版本号相同"会漏掉这种策略级变化**，这也是我们坚持"token + 路径"双轨而不是只比对 clientVersion 字符串的原因。

---

## 6. 模块四：同步产物形态与门禁

### 6.1 推荐：三层混合

| 层 | 内容 | 自动化 | 门禁 |
|---|---|---|---|
| ① 报告产物 | `upstream-report-<date>.md`：新 commit 决策矩阵、严重级、建议动作、新来源候选、版本漂移 | 全自动 | — |
| ② Draft PR | 报告作为 PR body/文件，分支 `sync/upstream-report-<date>`，label `upstream-sync`，自动指派仓库成员 | 全自动开出/更新；**永不 auto-merge** | **人工 review**（PR 合并 = 已阅确认 + 状态推进） |
| ③ 代码 PR | 人工依据报告实施的 Rust 改动（字段映射提案负责翻译方法） | 人工 | 现有 `build.yml`（cargo test + tsc + tauri build） |

### 6.2 为什么不是"自动开代码 PR"

- Python extractor → Rust 的翻译是**设计工作**（serde 结构、错误路径、流选择策略），无可行自动化；自动化生成代码 PR 只会产出需要返工的低质 diff（违反 AGENTS.md 开闭原则与谷歌规范）。
- 协议猜错会**阻塞全部用户的播放**，风险不对称——人工门禁是必要成本不是妥协。
- 报告 PR 的合并语义足够轻（一次点击），不会形成流程负担。

### 6.3 Draft PR 生命周期规则

1. 发现 RELEVANT commit → 开/更新 Draft PR；若已存在 open 的 `upstream-sync` PR → 直接 force-push 更新其分支并追加评论（附新 commit 列表），不重复开。
2. 人类合并报告 PR → 状态锁（§7）随之推进（bot 在合并后 commit）。**合并 = 已阅确认**，不是代码已同步。
3. 人类实施改动 → 独立代码 PR，PR body 引用报告 PR 编号。
4. 若连续 N 天（默认 14）报告 PR 无人动 → 追加提醒评论（可选）。

### 6.4 本地 CLI 输出

`python scripts/upstream-sync/sync.py`（无 `--ci`）：
- 终端打印与 CI 完全一致的报告（同函数生成）；
- 更新本地 `.sync/upstream-state.json`，开发者随代码改动一起 commit；
- 不 fetch 时加 `--offline`；指定上游 clone 路径用 `--repo ../yt-dlp`。

---

## 7. 模块五：失败与降级

### 7.1 状态锁格式（`.sync/upstream-state.json`）

```json
{
  "last_processed_sha": "69ea20006...",
  "upstream_version": "2026.07.04",
  "upstream_git_head": "997fa140840a08df3938b40da470c78049fef1f6",
  "last_run_at": "2026-08-06T03:17:00Z",
  "last_report": "upstream-report-2026-08-06.md"
}
```

- 由 bot commit 维护（`github-actions[bot]`，yt-dlp 更新 version.py 同款模式）；**仅在成功生成报告后推进**（at-least-once，重跑幂等：同 SHA → 无新 commit → 静默成功）。
- `last_processed_sha` 推进时机：CI 模式 = 报告 PR 合并后；本地模式 = 脚本运行成功后（由开发者随改动提交）。两者写同一个文件，语义一致。

### 7.2 失败模式表

| 失败模式 | 检测 | 行为 | 恢复 |
|---|---|---|---|
| 网络失败（API/`ls-remote`） | 异常 | sync job 失败 → on-failure job 开/更新 issue（label `upstream-sync`，先搜 open 去重） | 下次定时自动重试；本地 `--offline` 可照常跑 |
| GitHub API 限流（本地无 token） | 429 | 降级为纯 git（`ls-remote --tags` 不耗 API）；CI 自动带 `GITHUB_TOKEN` | 自动 |
| 上游 force-push / `last_sha` 不可达 | `git merge-base --is-ancestor` 失败 | 基线回退到最近 release tag（`git describe`），报告 WARN 标注"历史被改写，基线为 <tag>" | 人工确认一次即可 |
| 上游路径重构（如 youtube/ 目录改名） | Tier A 路径存在性检查失败 | 报告进 ERROR 段 + 开 issue；脚本不崩（保留上次报告模板） | 人工更新 tiers.yaml |
| 报告 PR 分支冲突 | push 失败 | 换新分支 `sync/upstream-report-<date>-r2`，自动关闭旧 PR | 自动 |
| 我方代码大幅重构导致 watchlist 失效 | 连续 N 次零命中但 Tier A 有改动 | 报告 WARN "watchlist 可能过时" | 人工维护（预计每次我方 extractor 大改时 ~15 分钟） |
| 上游动作太快（日多 commit） | commit 数 > 阈值（如 50） | 报告折叠为"批量处理"模式，只列最高严重级 | 人工 |

### 7.3 版本锁与可回滚性

- **版本锁**：状态锁内 `upstream_version` + `upstream_git_head` 双字段（对应 yt-dlp `version.py` 的 `__version__` + `RELEASE_GIT_HEAD`）；建议在 extractor 模块头加 `// upstream: yt-dlp <version> (<head>)` 注释，报告可自动核对"我们认领的版本 vs 上游最新"。
- **可回滚**：所有同步产物都在 git 历史里（报告 PR、代码 PR、状态锁），回滚 = `git revert`；`build.yml` 已对代码 PR 把关，报告 PR 合并不产生运行时代码，零风险。
- **保守模式**（可选 flag `--tag-filter`）：只对 release tag 之间的 diff 出报告，适合想以 release 粒度追踪的场景；默认 master 粒度（修复先到 master）。

### 7.4 与主 CI 的隔离

`upstream-sync.yml` 独立于 `build.yml`/`release.yml`，失败**永不**影响应用构建/发布；权限仅 `contents: write`，无额外 secret。

---

## 8. 模块六：新来源自动发现

### 8.1 机制

对 `last_sha..HEAD` 区间：

1. `git diff <last>..<HEAD> -- supportedsites.md` → 取 `^\+ - ` 行 = 新增支持站点条目；
2. 同时取新增文件 `yt_dlp/extractor/*.py` 及 `yt_dlp/extractor/*/__init__.py` 列表（排除 youtube/bilibili 已知项）；
3. 与 `music-sites.txt` 关键词表匹配（netease/qqmusic/kugou/kuwo/migu/spotify/soundcloud/bandcamp/deezer/tidal/audiomack/...），命中的进"候选新来源"段：条目行 + 新增 extractor 文件路径 + 对应 commit。

### 8.2 诚实边界

**发现是自动的，立项是人工的**：新增一个 Rust extractor = `extractor/<site>/` 全套 + adapter + 标识层扩展（`playback/model.rs`）+ 测试（extension-guide.md §1–5），这些无法自动生成。自动化输出的是"上游新增了 X 站支持，diff 在这"的情报卡片，决策（值不值得做、用户是否诉求）留在人。
去重：人工在 `scripts/upstream-sync/decisions.md` 记录"已评估/不采用/计划中"，脚本跳过已记录站点。

---

## 9. 文件/脚本落点清单

| 路径 | 内容 | 生命周期 |
|---|---|---|
| `.github/workflows/upstream-sync.yml` | 定时 + dispatch 触发、concurrency、bot commit、开 PR | 新增 |
| `scripts/upstream-sync/sync.py` | 唯一逻辑：fetch/diff → 分类 → watchlist 匹配 → 报告生成 → 状态推进 | 新增，~300 行 |
| `scripts/upstream-sync/tiers.yaml` | 路径分级 | 新增，低频维护 |
| `scripts/upstream-sync/watchlist.yaml` | 依赖符号清单（§5.1） | 新增，随我方 extractor 演进维护 |
| `scripts/upstream-sync/music-sites.txt` | 新来源发现关键词 | 新增，低频维护 |
| `scripts/upstream-sync/decisions.md` | 新来源立项决策记录（去重） | 新增，人工写 |
| `.sync/upstream-state.json` | 状态锁（bot/本地共同维护） | 新增，gitignore 排除后由 bot 提交 |
| `docs/upstream-reports/upstream-report-<date>.md` | 报告产物（PR 内） | 生成 |
| 现有 `build.yml` / `release.yml` | 代码 PR 门禁 | 不动 |

可选增强（不在首批）：`sync.py --embed-version` 在构建时把状态锁以 `include_str!` 编进二进制（`env!("CARGO_PKG_VERSION")` 同款思路），崩溃时可上报"我们基于 yt-dlp <version> 构建"。

---

## 10. 周期性运维成本评估

| 项 | 成本 | 频率 |
|---|---|---|
| CI 定时运行 | ~1–2 min/次 × 1/日 ≈ 每月 <1h CI 时间 | 自动 |
| 报告人工审阅 | 5–10 min/次 × RELEVANT 数（预计 2–6 次/月，YouTube 主导） | 人工 |
| 实际同步实施（改 client_version / 字段 / 参数） | 0.5–4 h/次；仅在 YouTube 反制变化时触发 | 人工，估计 1–3 次/月 |
| POT/解密类大改动（若被强制） | 数天级立项 | 罕见但必然发生（上游已现端倪） |
| tiers/watchlist 维护 | ~15 min，随我方 extractor 改动 | 低频 |
| 脚本自身维护 | 极低（纯 git/python stdlib + PyYAML/requests） | — |

**噪声控制自评**：路径过滤砍掉 900+ 站点（>90% commit）；YouTube 内部再按 watchlist 分级，真正开 PR 的只有 RELEVANT（预计每周 ≤1–2 条）。

---

## 11. 分阶段落地路线

| 阶段 | 内容 | 时长 | 验收 |
|---|---|---|---|
| P1 本地 CLI | `sync.py --offline` + tiers/watchlist v1 + 报告生成 | 1–2 天 | 对 `../yt-dlp` 现存 120 天 diff 跑出首份报告，人工核对分类准确率 |
| P2 CI 定时 | `upstream-sync.yml` + bot 状态提交 + 失败 issue 告警 | 半天 | 连续 3 次定时运行成功、幂等 |
| P3 Draft PR | 自动开/更新报告 PR + label + 合并后状态推进 | 半天 | 一次真实 RELEVANT 事件走通全链路 |
| P4 新来源发现 | supportedsites diff + music-sites 过滤 + decisions.md | 半天 | 复现一次新增站点条目（可先用历史 diff 测试） |
| P5 可选 | `--tag-filter` 保守模式、`--embed-version` | 按需 | — |

P1 先行且不依赖 CI，可立即验证分类规则质量（规则质量 = 本方案唯一需要调优的部分）。

---

## 12. 明确标注：人工 vs 自动（汇总）

**可全自动（无需人）：**
- 上游监测（定时 fetch、版本漂移检测、新 commit 枚举）
- 路径/关键词分类、严重级评定
- 报告生成、Draft PR 开/更新/提醒、状态锁 bot 推进、失败 issue 告警

**必须人工（自动化只做辅助）：**
- 报告审阅与合并（确认已阅）
- 任何 Rust 代码改动（字段映射、client 版本对齐、POT 立项）——交付物是情报不是代码
- tiers.yaml / watchlist.yaml / music-sites.txt 的维护
- 新来源立项决策

**风险敞口声明**：本方案最大的不确定性在"分类规则对上游实际演进的覆盖度"，已用路径存在性检查 + watchlist 失配警告 + 批量模式兜底；规则质量在 P1 阶段用历史 diff 实测校准，不进入自动 PR 之前先人工验证。
