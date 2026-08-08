"""repo_map：自动生成「我方代码地图」静态注入文本（零 LLM 注入面）。

用途：注入 analyze/translate/fix 的 system prompt，让 LLM 在第一次调用前
就知道我方代码的真实形态——消灭「模型不知道 context.rs 里没有那个常量」
之类的幻觉根因（见 CHROME_MAJOR_VERSION_RANGE 案例）。

生成内容（全部由脚本扫描，不经 LLM）：
1. 来源清单：sources.json 的 active 来源 + 各自 extractor/playback 文件
2. 常量/端点索引：提取各 extractor 里的关键常量与端点 URL
3. 上游↔我方映射：上游 client 键 → 我方 api.rs 行号（youtube 特化）

输出：SYNC_DIR/repo_map.md（提交入库，agent.py 注入）
"""
from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
SRC = ROOT / "src-tauri" / "src"
SYNC_DIR = ROOT / "tools" / "sync"
SOURCES_FILE = SYNC_DIR / "sources.json"
OUT_FILE = SYNC_DIR / "repo_map.md"


def load_sources() -> dict:
    if not SOURCES_FILE.exists():
        return {}
    return json.loads(SOURCES_FILE.read_text())


def active_sources() -> list[tuple[str, dict]]:
    sources = load_sources()
    return [
        (site, cfg) for site, cfg in sources.items()
        if cfg.get("status") in ("in_progress", "adopted")
    ]


def _extract_constants(text: str, max_items: int = 15) -> list[str]:
    """提取 const/static 定义行（前 max_items 条）。"""
    out = []
    for m in re.finditer(
        r"^\s*(?:pub\s+)?(?:const|static)\s+([A-Z_][A-Z0-9_]*)\s*:",
        text, re.M):
        out.append(m.group(1))
        if len(out) >= max_items:
            break
    return out


def _extract_urls(text: str, max_items: int = 8) -> list[str]:
    """提取 https:// 端点（截断显示）。"""
    out = []
    for m in re.finditer(r"https://[^\s\"')]+", text):
        u = m.group(0)
        if u not in out:
            out.append(u[:90])
        if len(out) >= max_items:
            break
    return out


def gen_sources_section() -> str:
    lines = ["## 来源清单（sources.json）", ""]
    for site, cfg in active_sources():
        lines.append(
            f"- `{site}`：prefix=`{cfg.get('prefix')}`, rust_name=`{cfg.get('rust_name')}`, "
            f"has_search={cfg.get('has_search', True)}, status={cfg.get('status')}")
    return "\n".join(lines) + "\n"


def gen_extractor_index() -> str:
    lines = ["## extractor 代码地图", ""]
    for extractor_dir in sorted((SRC / "extractor").iterdir()):
        if not extractor_dir.is_dir():
            continue
        site = extractor_dir.name
        lines.append(f"### {site}/")
        for f in sorted(extractor_dir.glob("*.rs")):
            text = f.read_text(errors="ignore")
            consts = _extract_constants(text)
            urls = _extract_urls(text)
            lines.append(f"- `{site}/{f.name}`: {len(text.splitlines())} 行")
            if consts:
                lines.append(f"  - 常量: {', '.join(consts)}")
            if urls:
                lines.append(f"  - 端点: {', '.join(urls)}")
    return "\n".join(lines) + "\n"


def gen_playback_index() -> str:
    lines = ["## playback adapter 地图", ""]
    playback = SRC / "playback"
    for f in sorted(playback.glob("*.rs")):
        if f.name in ("mod.rs", "catalog.rs", "service.rs", "spool.rs",
                      "trace.rs", "resolver.rs", "search.rs", "runtime.rs", "model.rs"):
            continue  # 核心管线不列，adapter 才有来源差异
        text = f.read_text(errors="ignore")
        lines.append(f"- `playback/{f.name}`: {len(text.splitlines())} 行")
    return "\n".join(lines) + "\n"


def gen_youtube_mapping() -> str:
    """上游 client 键 ↔ 我方 api.rs 行号（youtube 特化，最常同步的面）。"""
    lines = ["## 上游↔我方映射（youtube client 配置）", ""]
    api_rs = SRC / "extractor" / "youtube" / "api.rs"
    if not api_rs.exists():
        return ""
    text = api_rs.read_text(errors="ignore")
    # 我方 client 条目：client_name 出现在哪一行
    for m in re.finditer(r'client_name:\s*"([A-Z_]+)"', text):
        name = m.group(1)
        line = text[: m.start()].count("\n") + 1
        # 找该条目附近的 client_version
        seg = text[m.start(): m.start() + 600]
        vm = re.search(r'client_version:\s*"([^"]+)"', seg)
        ver = vm.group(1) if vm else "?"
        lines.append(f"- 我方 `{name}`（api.rs:{line}）client_version=`{ver}`")
    lines.append("")
    lines.append("上游对应：`yt_dlp/extractor/youtube/_base.py` 的 `INNERTUBE_CLIENTS` 键（web_music→WEB_REMIX 等）。")
    return "\n".join(lines) + "\n"


def gen_repo_map() -> str:
    parts = [
        "# repo_map（自动生成 by tools/sync/repo_map.py，勿手改）",
        "",
        "> 这是 music_demo 代码的静态地图。翻译上游改动前先读本节，",
        "> 确认我方对应文件的真实形态（常量名/端点/行号），不要凭上游命名猜我方结构。",
        "",
        gen_sources_section(),
        gen_extractor_index(),
        gen_playback_index(),
        gen_youtube_mapping(),
    ]
    return "\n".join(parts)


def main() -> None:
    OUT_FILE.write_text(gen_repo_map())
    print(f"[repo_map] wrote {OUT_FILE} ({OUT_FILE.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
