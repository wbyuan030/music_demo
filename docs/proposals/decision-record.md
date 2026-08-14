# Decision Record: yt-dlp 上游同步方案

> 状态：brainstorm 定稿（未实施，不改任何代码）
> 取代 `upstream-sync-workflow.md`（关键词过滤方案，已否决）与 `ytdlp-bridge-vs-static.md`（桥接打包方案，已否决）。
> 关联：[architecture.md](../architecture.md)、[contracts.md](../contracts.md)、[extension-guide.md](../extension-guide.md)

---

## 1. 核心策略

**纯参考 + agent 翻译 + 测试驱动**：yt-dlp 源码作 dev-time 参考，不打包、不引入 Python 运行时。上游更新时 agent 语义分析 diff，翻译成 Rust extractor 改动，测试是唯一验收过滤器。

否决方案：
- **关键词/路径/规则匹配**（upstream-sync-workflow.md）：启发式必漏，策略级变化无法被 token 匹配捕获。
- **yt-dlp 子进程桥接打包**（ytdlp-bridge-vs-static.md）：+30-50MB 体积、HLS/DASH 不可播、搜索串行枚举延迟、macOS 签名问题、移动端不可行。

---

## 2. 三条同步线

### 2.1 YouTube 字段同步（字段层）

从 yt-dlp `_base.py` 的 `INNERTUBE_CLIENTS` 字面量 dict 用 Python `ast` 模块提取（不 import 执行），生成 `generated.rs` 编译期常量。

| 评级 | 含义 | 项目 |
|---|---|---|
| A（纯数据可自动提取） | clientName/Version、sdkVersion、osName/Version、INNERTUBE_CONTEXT_CLIENT_NAME、PO policy 布尔、端点模板、player.js 变体路径表 | 主体收益 |
| B（逻辑需人工适配） | signatureTimestamp 提取、visitorData 路径、ytInitialPlayerResponse 解析 | 标记人工 |
| C（无法复用） | innertube API key（上游已删，本地维护）、s/n 签名求解（jsc 架构）、po_token 求解（BotGuard） | 不做自动同步，锚点检测预警 |

载体：`scripts/sync_ytdlp.py`（手动/CI 触发）-> `yt_innertube_snapshot.json`（提交）-> `generated.rs`（提交，`@generated` 头）。漂移检测：`--check` 对 AST 字面量 diff，差异非 0 退出码。防漂移兜底：`api.rs` 契约测试断言序列化后的 clientName/Version 等于 `generated.rs` 常量。

### 2.2 新来源自动发现（来源层）

- 候选清单生成器调 yt-dlp `list_extractor_classes()` 枚举 API，按音乐关键词评分过滤（实测 1731 条中 ~87 个候选）。
- `gen_adapter.py` 读 `sources.json`（人工填 3 个决策：prefix、rust_name、has_search），一键生成 extractor 骨架 + adapter + 标识层 + 注册 + 前缀表 + 测试桩。
- **marker 块机制**：`// ==== sync-generated:begin <name> ====` ... `end ====`，生成器幂等重写块内内容，块外人工改动不受影响。
- 人工残留：实现 extractor 协议代码 + 录制 fixture + 跑一次 smoke。接入成本从"跨 7 文件 2-3 天"降到"1 文件 + 1 晚"。

### 2.3 Agent 驱动同步工作流（流程层）

六阶段：detect -> vendor -> analyze -> translate -> test -> PR。

| 翻译分型 | 上游改动形态 | 动作 | 自动化 |
|---|---|---|---|
| T1 纯数据 | 常量、端点、正则字面量 | 复用 AST 提取脚本更新 generated.rs | 全自动 |
| T2 逻辑 | 签名算法、解析路径、流选择 | agent 手写 Rust 适配，难度高标记人工 | 部分自动 |
| T3 新来源 | 新增 extractor 文件 | 复用 gen_adapter.py 生成骨架 | 骨架自动，协议实现不自动 |
| T4 无关 | 其他 900+ 站点 | 不进 PR，计入摘要 | 全自动 |

执行器：自研 Python 脚本（`tools/sync/`），LLM 只做理解与生成，确定性循环在脚本侧。安全：LLM 输出 JSON schema 校验 + 补丁路径白名单（仅 `src-tauri/src/`、`tools/sync/`、`.sync/`）。

GitHub 托管变体：不 vendor 进 repo（PR 只含 Rust 翻译），锁文件 `.sync/upstream-state.json` 记上游 sha；oracle 用本地录制的黄金 fixture 离线比对。

---

## 3. 测试验收标准（测试是唯一过滤器）

| 层 | 内容 | 离线 | 判定 |
|---|---|---|---|
| 1 | `cargo test`：契约测试（请求体序列化断言、ID 前缀 round-trip、generated.rs 一致性） | ✅ | 必绿，阻塞 |
| 2 | oracle 差分：固定 fixture URL 下 yt-dlp `-J` vs music-cli 比对 | ❌ | 警告不阻塞 |
| 3 | 网络 smoke：真实站点解析 + validate_url 200/206 | ❌ | 必过（夜间/手动） |

新来源接入完成 = mock 行为测试（搜索 + 解析 + 缓存）从红变绿。生成器写入的模板测试默认红（`assert!(false)` 占位），禁止 `#[ignore]` 绕过。

---

## 4. 落地路径

| 阶段 | 内容 | 周期 |
|---|---|---|
| P0 | `tools/sync/sync.py`（detect/vendor）+ 锁文件 | 1 天 |
| P1 | T1 试点：AST 提取 + generated.rs + 契约测试 | 2-3 天 |
| P2 | agent 编排（analyze/translate）+ oracle 差分 | 3-5 天 |
| P3 | 修复循环 + PR 自动化 + CI cron | 2-3 天 |
| P4 | T3 新来源骨架生成接入 workflow | 2 天 |

---

## 5. 风险与诚实边界

- **T2 高难逻辑（签名/POT）不自动**：yt-dlp `jsc/`+`pot/` 架构无法移植到 Rust，标记人工。
- **T3 extractor 协议实现不自动**：新来源骨架 + 注册全自动，逆向实现仍需人工（每个 300-500 行 Python）。
- **上游大重构**：agent 标记 T2 人工，PR 转 draft。
- **API key 无上游源**：yt-dlp 2024-07 已删除硬编码 key，music_demo 保留本地常量。失效时升级方案为 `ensure_visitor_session` 顺带解析页面 ytcfg。

---

## 6. 已否决方案摘要

| 方案 | 否决理由 |
|---|---|
| 关键词/路径/规则过滤 | 启发式必漏；策略级变化（如 POT enforcement）不被 token 匹配捕获 |
| yt-dlp 子进程桥接 | +30-50MB；HLS/DASH 不可播；搜索串行枚举延迟；macOS 签名；移动端不可行 |
| 构建期 codegen（build.rs） | 构建期硬依赖 `../yt-dlp` 外部路径，CI 机器未必有 |
| 运行时 JSON 加载 | 低频变动不值得运行时 IO；Tauri 打包资源嵌入复杂 |
