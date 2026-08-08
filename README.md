# music_demo

一个半成品桌面音乐播放器：搜索并播放 YouTube / Bilibili 音频，可解析微信公众号文章里插入的音乐。本地缓存 + 流式播放（边下边播），最近播放与收藏持久化到本地数据库。

## 技术栈

- **桌面壳**：Tauri v2
- **前端**：React 19 + TypeScript + Vite + Zustand + Tailwind CSS
- **后端**：Rust（tokio + reqwest + rodio 播放 + native_db 持久化）
- **Cargo workspace**：`src-tauri`（应用库 `app_lib`）、`cli`（`music-cli` 调试 CLI）

## 文档入口

| 文档 | 内容 |
|---|---|
| [docs/architecture.md](docs/architecture.md) | 架构总览：模型、原则、模块地图、运行时边界、变更指南（推荐先读） |
| [docs/contracts.md](docs/contracts.md) | command / event 契约、播放状态同步模型（SSOT）、数据持久化与兼容约束 |
| [docs/extension-guide.md](docs/extension-guide.md) | 新增音乐来源的完整指南 |
| [docs/observability.md](docs/observability.md) | 播放 trace、欠载探针、前端日志转发 |

## 常用命令

```bash
# 启动开发环境（前端 dev server + Tauri 应用）
npm install
npm run tauri dev          # 等价于 npx tauri dev

# 测试（全部 workspace 成员）
cargo test --workspace

# 前端生产构建
npm run build

# CLI：搜索 / 播放清单 / 信息 / 下载
cargo run -p music-cli -- search "<keyword>" --source all
cargo run -p music-cli -- manifest <youtube-video-id>      # YouTube 音频清单
cargo run -p music-cli -- info <youtube-video-id>          # YouTube 视频信息
cargo run -p music-cli -- manifest-bili <bvid>             # Bilibili 音频清单
cargo run -p music-cli -- download <youtube-video-id> [-o out.audio]
```
