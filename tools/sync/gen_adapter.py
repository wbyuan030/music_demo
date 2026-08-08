"""T3 新来源骨架生成器：一键生成接入所需的所有机械代码。

读 tools/sync/sources.json（人工维护注册表），对 status == "in_progress"
的条目生成/更新：

1. src-tauri/src/extractor/<site>/     — 协议桩（mod.rs + search.rs + player.rs，返回 Unimplemented）
2. src-tauri/src/playback/<site>.rs     — 完整 adapter（SearchProvider + PlaybackResolver）
3. src-tauri/src/playback/model.rs      — SourceKind/TrackId/SourceRef（marker 块内重写）
4. src-tauri/src/types.rs               — TrackMeta from/to_source_ref（marker 块内追加）
5. src-tauri/src/playback/search.rs     — track_to_entry 前缀表（marker 块内追加）
6. src-tauri/src/playback/runtime.rs    — 注册行（marker 块内追加）
7. 契约测试骨架（内嵌 adapter 文件 #[cfg(test)]）

marker 块：
    // ==== sync-generated:begin <name> ====
    ...
    // ==== sync-generated:end <name> ====
生成器每次整体重写块内内容（幂等），块外人工改动不受影响。

sources.json 格式：
{
  "soundcloud": {
    "prefix": "sc",          // ≤8 字符小写，全局唯一（yt:/bili: 已占用）
    "rust_name": "Soundcloud",
    "has_search": true,      // 是否有搜索能力（false = resolver-only）
    "field": "url",          // SourceRef 字段名：url / id / track_id ...
    "status": "in_progress"  // candidate → in_progress → adopted
  }
}
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
SRC = ROOT / "src-tauri" / "src"
PLAYBACK = SRC / "playback"
EXTRACTOR = SRC / "extractor"
SYNC_DIR = ROOT / "tools" / "sync"
SOURCES_FILE = SYNC_DIR / "sources.json"

MARKERS = {
    "source_kinds": (
        "// ==== sync-generated:begin source_kinds ====",
        "// ==== sync-generated:end source_kinds ====",
    ),
    "source_kind_as_str": (
        "// ==== sync-generated:begin source_kind_as_str ====",
        "// ==== sync-generated:end source_kind_as_str ====",
    ),
    "track_id_display": (
        "// ==== sync-generated:begin track_id_display ====",
        "// ==== sync-generated:end track_id_display ====",
    ),
    "track_id_fromstr": (
        "// ==== sync-generated:begin track_id_fromstr ====",
        "// ==== sync-generated:end track_id_fromstr ====",
    ),
    "source_ref": (
        "// ==== sync-generated:begin source_ref ====",
        "// ==== sync-generated:end source_ref ====",
    ),
    "source_ref_kind": (
        "// ==== sync-generated:begin source_ref_kind ====",
        "// ==== sync-generated:end source_ref_kind ====",
    ),
    "track_meta": (
        "// ==== sync-generated:begin track_meta ====",
        "// ==== sync-generated:end track_meta ====",
    ),
    "track_meta_reverse": (
        "// ==== sync-generated:begin track_meta_reverse ====",
        "// ==== sync-generated:end track_meta_reverse ====",
    ),
    "track_to_entry": (
        "// ==== sync-generated:begin track_to_entry ====",
        "// ==== sync-generated:end track_to_entry ====",
    ),
    "track_to_entry_ref": (
        "// ==== sync-generated:begin track_to_entry_ref ====",
        "// ==== sync-generated:end track_to_entry_ref ====",
    ),
    "runtime_register": (
        "// ==== sync-generated:begin runtime_register ====",
        "// ==== sync-generated:end runtime_register ====",
    ),
    "runtime_register_search": (
        "// ==== sync-generated:begin runtime_register_search ====",
        "// ==== sync-generated:end runtime_register_search ====",
    ),
    "playback_mod_decl": (
        "// ==== sync-generated:begin playback_mod_decl ====",
        "// ==== sync-generated:end playback_mod_decl ====",
    ),
    "extractor_mod_decl": (
        "// ==== sync-generated:begin extractor_mod_decl ====",
        "// ==== sync-generated:end extractor_mod_decl ====",
    ),
    "runtime_use": (
        "// ==== sync-generated:begin runtime_use ====",
        "// ==== sync-generated:end runtime_use ====",
    ),
}

RESERVED_PREFIXES = {"yt", "bili"}  # wechat 无前缀


def load_sources() -> dict:
    if not SOURCES_FILE.exists():
        SOURCES_FILE.write_text(json.dumps({}, indent=2) + "\n")
    return json.loads(SOURCES_FILE.read_text())


def active_sources(sources: dict) -> list[tuple[str, dict]]:
    """status in (in_progress, adopted) 且有 prefix/rust_name 的条目。"""
    out = []
    for site, cfg in sources.items():
        if cfg.get("status") not in ("in_progress", "adopted"):
            continue
        if not cfg.get("prefix") or not cfg.get("rust_name"):
            print(f"[gen] skip {site}: 缺 prefix/rust_name")
            continue
        out.append((site, cfg))
    return sorted(out, key=lambda x: x[0])


def replace_marker(path: Path, name: str, content: str) -> bool:
    """替换 marker 块内容。块必须已存在（首次放置见 marker 放置指南）。返回是否修改。"""
    begin, end = MARKERS[name]
    text = path.read_text()
    if begin not in text or end not in text:
        print(f"[gen] 缺少 marker 块 {name} in {path}，需先手动放置（见下方放置指南）")
        print(f"  {begin}")
        print(f"  ...")
        print(f"  {end}")
        return False
    start = text.index(begin)
    stop = text.index(end) + len(end)
    block = f"{begin}\n{content}\n{end}"
    new_text = text[:start] + block + text[stop:]
    if new_text != text:
        path.write_text(new_text)
        return True
    return False


# ── 各文件块生成 ──────────────────────────────────────────────────────

def gen_source_kinds(active: list[tuple[str, dict]]) -> str:
    return "\n".join(
        f"    {cfg['rust_name']}," for _, cfg in active)


def gen_source_kind_as_str(active: list[tuple[str, dict]]) -> str:
    return "\n".join(
        f"            Self::{cfg['rust_name']} => \"{cfg['rust_name'].lower()}\"," for _, cfg in active)


def gen_track_id_display(active: list[tuple[str, dict]]) -> str:
    return "\n".join(
        f"            SourceKind::{cfg['rust_name']} => write!(f, \"{cfg['prefix']}:{{}}\", self.remote_id),"
        for _, cfg in active)


def gen_track_id_fromstr(active: list[tuple[str, dict]]) -> str:
    lines = []
    for _, cfg in active:
        lines.append(f"""        if let Some(remote_id) = value.strip_prefix("{cfg['prefix']}:") {{
            if !remote_id.is_empty() {{
                return Ok(Self {{
                    source: SourceKind::{cfg['rust_name']},
                    remote_id: remote_id.to_string(),
                }});
            }}
        }}""")
    return "\n".join(lines)


def gen_source_ref(active: list[tuple[str, dict]]) -> str:
    return "\n".join(
        f"    {cfg['rust_name']} {{ {cfg.get('field', 'id')}: String }}," for _, cfg in active)


def gen_source_ref_kind(active: list[tuple[str, dict]]) -> str:
    return "\n".join(
        f"            Self::{cfg['rust_name']} {{ .. }} => SourceKind::{cfg['rust_name']},"
        for _, cfg in active)


def gen_track_meta(active: list[tuple[str, dict]]) -> str:
    lines = []
    for _, cfg in active:
        field = cfg.get("field", "id")
        lines.append(
            f"            crate::playback::model::SourceRef::{cfg['rust_name']} {{ {field} }} => Self {{\n"
            f'                source: "extractor".to_string(),\n'
            f'                value: MetaValue::Extractor(format!("{cfg["prefix"]}:{{}}", {field})),\n'
            f"            }},"
        )
    # to_source_ref 的 strip_prefix 分支
    return "\n".join(lines)


def gen_track_meta_reverse(active: list[tuple[str, dict]]) -> str:
    lines = []
    for _, cfg in active:
        field = cfg.get("field", "id")
        lines.append(f"""                if let Some({field}) = id.strip_prefix("{cfg['prefix']}:") {{
                    return Some(crate::playback::model::SourceRef::{cfg['rust_name']} {{
                        {field}: {field}.to_string(),
                    }});
                }}""")
    return "\n".join(lines)


def gen_track_to_entry(active: list[tuple[str, dict]]) -> str:
    lines = []
    for _, cfg in active:
        lines.append(
            f"        SourceKind::{cfg['rust_name']} => track.id.strip_prefix(\"{cfg['prefix']}:\")?,"
        )
    return "\n".join(lines)


def gen_track_to_entry_ref(active: list[tuple[str, dict]]) -> str:
    refs = []
    for _, cfg in active:
        field = cfg.get("field", "id")
        refs.append(
            f"        SourceKind::{cfg['rust_name']} => SourceRef::{cfg['rust_name']} {{\n"
            f"            {field}: remote_id.to_string(),\n"
            f"        }},"
        )
    return "\n".join(refs)


def gen_runtime_register(active: list[tuple[str, dict]]) -> str:
    lines = []
    for _, cfg in active:
        lines.append(f"        registry.register({cfg['rust_name']}Source);")
    return "\n".join(lines)


def gen_runtime_register_search(active: list[tuple[str, dict]]) -> str:
    lines = []
    for _, cfg in active:
        if cfg.get("has_search", True):
            lines.append(f"        search.register({cfg['rust_name']}Source);")
    return "\n".join(lines)


# ── 新文件生成 ────────────────────────────────────────────────────────

ADAPTER_TMPL = """use async_trait::async_trait;

use crate::extractor::{{context::ExtractorContext, model::PlaybackManifest}};

use super::{{
    model::{{PlayableEntry, SourceKind, SourceRef}},
    resolver::{{PlaybackError, PlaybackResolver}},
    search::{{track_to_entry, SearchProvider}},
}};

/// {rust_name} 来源适配器：搜索 + 播放解析共用同一套来源抽象。
pub struct {rust_name}Source;

#[async_trait]
impl SearchProvider for {rust_name}Source {{
    fn source(&self) -> SourceKind {{
        SourceKind::{rust_name}
    }}

    async fn search(
        &self,
        keyword: &str,
        context: &ExtractorContext,
    ) -> Result<Vec<PlayableEntry>, PlaybackError> {{
        let tracks = crate::extractor::{site}::search::search_music(context, keyword, None).await?;
        Ok(tracks
            .into_iter()
            .filter_map(|track| track_to_entry(track, SourceKind::{rust_name}))
            .collect())
    }}
}}

#[async_trait]
impl PlaybackResolver for {rust_name}Source {{
    fn source(&self) -> SourceKind {{
        SourceKind::{rust_name}
    }}

    async fn resolve(
        &self,
        entry: &PlayableEntry,
        context: &ExtractorContext,
    ) -> Result<PlaybackManifest, PlaybackError> {{
        let SourceRef::{rust_name} {{ {field} }} = &entry.source_ref else {{
            return Err(PlaybackError::NoResolver(
                SourceKind::{rust_name}.as_str().to_string(),
            ));
        }};
        crate::extractor::{site}::player::get_manifest(context, {field}).await.map_err(Into::into)
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;
    use crate::extractor::model::Track;

    #[test]
    fn track_id_prefix_is_stable() {{
        let id = format!("{prefix}:abc123");
        let track = Track {{
            id: id.clone(),
            title: "t".into(),
            artists: vec![],
            album: None,
            duration_ms: None,
            artwork: vec![],
        }};
        let entry = track_to_entry(track, SourceKind::{rust_name}).expect("entry");
        assert_eq!(entry.view.id, id);
    }}

    #[test]
    fn resolver_rejects_wrong_source() {{
        let entry = PlayableEntry {{
            view: crate::types::TrackView::new("x".into(), "y".into(), "".into(), 0.0, "yt:abc".into()),
            source_ref: SourceRef::Youtube {{ video_id: "abc".into() }},
        }};
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {{
            let ctx = ExtractorContext::with_client(reqwest::Client::new());
            {rust_name}Source.resolve(&entry, &ctx).await
        }});
        assert!(matches!(result, Err(PlaybackError::NoResolver(_))));
    }}

    /// 行为测试 3（接入完成前红）：resolve 能拿到流 → PlaybackService 流式下载
    /// 到临时文件 → 内容一致 → 二次加载命中缓存。
    /// 复用 playback/service.rs 的 downloads_once_and_reuses_stable_cache 模式：
    /// 需要 manifest 的 streams[0].url 指向本地 mock 音频（录一段真实音频 fixture）。
    #[tokio::test(flavor = "multi_thread")]
    async fn resolves_then_spools_and_caches() {{
        // TODO(接入完成): 参考 service.rs 同名测试——
        //   1. fixtures/audio.mp3 录制真实音频；
        //   2. player.json 的 streams[0].url 指向本地 mock 音频地址；
        //   3. 走 PlaybackService::load_track_source → 断言 Progressive 读取内容一致；
        //   4. 断言缓存原子提交 + 二次加载 TrackSource::File 命中。
        // 未实现前保持红：防止骨架合入但播放链路没验证。
        assert!(false, "行为测试 3 未实现：完成 {site} 协议 + 录制音频 fixture 后启用");
    }}
}}
"""

EXTRACTOR_MOD_TMPL = """pub mod player;
pub mod search;
"""

EXTRACTOR_SEARCH_TMPL = """use crate::extractor::context::ExtractorContext;
use crate::extractor::model::Track;
use crate::extractor::protocol::ExtractError;

/// {site} 搜索端点（默认生产 URL；测试用 options.endpoints 覆盖）。
pub const SEARCH_ENDPOINT: &str = "{search_url}";

/// {site} 搜索。TODO: 实现协议（参考 ../yt-dlp/yt_dlp/extractor/{ytdlp_file}）。
/// 实现后必须补行为测试（见下方 tests）。
pub async fn search_music(
    ctx: &ExtractorContext,
    keyword: &str,
    _limit: Option<u32>,
) -> Result<Vec<Track>, ExtractError> {{
    let url = ctx.options.endpoint("search", SEARCH_ENDPOINT);
    Err(ExtractError::ExtractionFailed(format!(
        "{site} search not implemented: {{url}}",
    )))
}}

#[cfg(test)]
mod tests {{
    use super::*;
    use tokio::io::{{AsyncReadExt, AsyncWriteExt}};
    use crate::extractor::context::ExtractorOptions;

    /// 行为测试 1：来源能搜索（mock，CI 门禁）。
    /// 本地 mock 搜索端点返回录制的真实响应，走真实 HTTP 代码路径。
    #[tokio::test]
    async fn can_search_via_mock() {{
        // 1. 录制真实搜索响应到 fixtures/search.json
        // 2. 本地 mock server 返回它
        let raw = include_str!("fixtures/search.json");
        assert_ne!(raw.trim(), "{{}}", "fixtures/search.json 未录制：先跑 record_fixture.py");

        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let addr = listener.local_addr().unwrap();
        let body = raw.to_string();
        let server = tokio::spawn(async move {{
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 4096];
            let _ = socket.read(&mut buf).await;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {{}}\r\nConnection: close\r\n\r\n{{}}",
                body.len(),
                body
            );
            socket.write_all(resp.as_bytes()).await.unwrap();
        }});

        let ctx = ExtractorContext::with_client(
                reqwest::Client::builder().no_proxy().build().unwrap(),
            )
            .with_options(ExtractorOptions::default().with_endpoint("search", &format!("http://{{}}", addr)));
        // TODO: let tracks = search_music(&ctx, "keyword", None).await.unwrap();
        // assert!(!tracks.is_empty());
        // assert!(tracks[0].id.starts_with("{prefix}:"));
        let _ = server;
    }}

    /// 真实流量 smoke：验证真实站点当前可用（夜间/发布前 job 跑，CI 默认跳过）。
    /// 红 = 站点协议变了/反爬/被限流 → 录新 fixture 更新 mock 测试。
    #[tokio::test]
    #[ignore = "真实网络，由 nightly smoke job 运行"]
    async fn real_site_search_smoke() {{
        let ctx = ExtractorContext::with_client(reqwest::Client::new());
        // TODO: let tracks = search_music(&ctx, "test", Some(1)).await.unwrap();
        // assert!(!tracks.is_empty(), "真实搜索返回空");
        let _ = ctx;
    }}
}}
"""

EXTRACTOR_PLAYER_TMPL = """use crate::extractor::context::ExtractorContext;
use crate::extractor::model::PlaybackManifest;
use crate::extractor::protocol::ExtractError;

/// {site} 播放解析端点（默认生产 URL；测试用 options.endpoints 覆盖）。
pub const PLAYER_ENDPOINT: &str = "{player_url}";

/// {site} 播放解析。TODO: 实现协议（参考 ../yt-dlp/yt_dlp/extractor/{ytdlp_file}）。
/// 实现后必须补行为测试（见下方 tests）。
pub async fn get_manifest(
    ctx: &ExtractorContext,
    _id: &str,
) -> Result<PlaybackManifest, ExtractError> {{
    let url = ctx.options.endpoint("player", PLAYER_ENDPOINT);
    Err(ExtractError::ExtractionFailed(format!(
        "{site} player not implemented: {{url}}",
    )))
}}

#[cfg(test)]
mod tests {{
    use super::*;
    use tokio::io::{{AsyncReadExt, AsyncWriteExt}};
    use crate::extractor::context::ExtractorOptions;

    /// 行为测试 2：来源能解析出可播放流（mock，CI 门禁）。
    /// 本地 mock 播放端点返回录制的真实响应（含直链 URL）。
    #[tokio::test]
    async fn can_resolve_manifest_via_mock() {{
        let raw = include_str!("fixtures/player.json");
        assert_ne!(raw.trim(), "{{}}", "fixtures/player.json 未录制：先跑 record_fixture.py");

        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let addr = listener.local_addr().unwrap();
        let body = raw.to_string();
        let server = tokio::spawn(async move {{
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 4096];
            let _ = socket.read(&mut buf).await;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {{}}\r\nConnection: close\r\n\r\n{{}}",
                body.len(),
                body
            );
            socket.write_all(resp.as_bytes()).await.unwrap();
        }});

        let ctx = ExtractorContext::with_client(
                reqwest::Client::builder().no_proxy().build().unwrap(),
            )
            .with_options(ExtractorOptions::default().with_endpoint("player", &format!("http://{{}}", addr)));
        // TODO: let manifest = get_manifest(&ctx, "id").await.unwrap();
        // assert!(!manifest.streams.is_empty());
        // assert!(manifest.streams[0].url.starts_with("http"));
        let _ = server;
    }}

    /// 真实流量 smoke：真实解析一个已知 ID（夜间/发布前 job 跑）。
    #[tokio::test]
    #[ignore = "真实网络，由 nightly smoke job 运行"]
    async fn real_site_player_smoke() {{
        let ctx = ExtractorContext::with_client(reqwest::Client::new());
        // TODO: let manifest = get_manifest(&ctx, "<已知 ID>").await.unwrap();
        // assert!(!manifest.streams.is_empty(), "真实播放解析返回空流");
        let _ = ctx;
    }}
}}
"""


def gen_source_files(site: str, cfg: dict) -> None:
    rust = cfg["rust_name"]
    prefix = cfg["prefix"]
    field = cfg.get("field", "id")
    ytdlp_file = cfg.get("ytdlp_file", f"{site}.py")
    search_url = cfg.get("search_url", f"https://{site}.example.com/api/search")
    player_url = cfg.get("player_url", f"https://{site}.example.com/api/player")
    d = EXTRACTOR / site
    d.mkdir(parents=True, exist_ok=True)
    (d / "fixtures").mkdir(exist_ok=True)
    # 占位 fixture：include_str! 需要文件存在才能编译；实现者用
    # record_fixture.py 覆盖为真实响应
    for name in ("search.json", "player.json"):
        fp = d / "fixtures" / name
        if not fp.exists():
            fp.write_text("{}\n")
    # 只写不存在的文件：协议已实现（人工/agent 写完）后生成器绝不覆盖
    # （幂等 + 不破坏人工改动的关键约定）
    files = {
        d / "mod.rs": EXTRACTOR_MOD_TMPL.format(site=site),
        d / "search.rs": EXTRACTOR_SEARCH_TMPL.format(
            site=site, ytdlp_file=ytdlp_file, prefix=prefix, search_url=search_url),
        d / "player.rs": EXTRACTOR_PLAYER_TMPL.format(
            site=site, ytdlp_file=ytdlp_file, player_url=player_url),
        PLAYBACK / f"{site}.rs": ADAPTER_TMPL.format(
            site=site, rust_name=rust, prefix=prefix, field=field),
    }
    for path, content in files.items():
        if not path.exists():
            path.write_text(content)
            print(f"[gen] wrote {path.relative_to(ROOT)}")
        else:
            print(f"[gen] skip existing {path.relative_to(ROOT)}（人工实现，不覆盖）")


def gen_playback_mod_decl(active: list[tuple[str, dict]]) -> str:
    return "\n".join(f"pub mod {site};" for site, _ in active)


def gen_runtime_use(active: list[tuple[str, dict]]) -> str:
    return "\n".join(
        f"    {site}::{cfg['rust_name']}Source," for site, cfg in active)


def gen_extractor_mod_decl(active: list[tuple[str, dict]]) -> str:
    return "\n".join(f"pub mod {site};" for site, _ in active)


def gen_all(sources: dict) -> None:
    active = active_sources(sources)
    print(f"[gen] {len(active)} active source(s): {[s for s, _ in active]}")
    if not active:
        print("[gen] nothing to do")
        return
    # 1) extractor 桩 + adapter 文件
    for site, cfg in active:
        gen_source_files(site, cfg)
    # 2) marker 块改写（幂等）
    model_path = PLAYBACK / "model.rs"
    replace_marker(model_path, "source_kinds", gen_source_kinds(active))
    replace_marker(model_path, "source_kind_as_str", gen_source_kind_as_str(active))
    replace_marker(model_path, "track_id_display", gen_track_id_display(active))
    replace_marker(model_path, "track_id_fromstr", gen_track_id_fromstr(active))
    replace_marker(model_path, "source_ref", gen_source_ref(active))
    replace_marker(model_path, "source_ref_kind", gen_source_ref_kind(active))
    types_path = SRC / "types.rs"
    replace_marker(types_path, "track_meta", gen_track_meta(active))
    replace_marker(types_path, "track_meta_reverse", gen_track_meta_reverse(active))
    search_path = PLAYBACK / "search.rs"
    replace_marker(search_path, "track_to_entry", gen_track_to_entry(active))
    replace_marker(search_path, "track_to_entry_ref", gen_track_to_entry_ref(active))
    runtime_path = PLAYBACK / "runtime.rs"
    replace_marker(runtime_path, "runtime_register", gen_runtime_register(active))
    replace_marker(runtime_path, "runtime_register_search", gen_runtime_register_search(active))
    replace_marker(runtime_path, "runtime_use", gen_runtime_use(active))
    mod_path = PLAYBACK / "mod.rs"
    replace_marker(mod_path, "playback_mod_decl", gen_playback_mod_decl(active))
    replace_marker(EXTRACTOR / "mod.rs", "extractor_mod_decl", gen_extractor_mod_decl(active))
    print("[gen] marker blocks updated")


def main() -> None:
    p = argparse.ArgumentParser(prog="gen_adapter.py")
    p.add_argument("--check", action="store_true",
                   help="只校验 sources.json 与 marker 块状态，不写文件")
    args = p.parse_args()
    sources = load_sources()
    if args.check:
        active = active_sources(sources)
        for site, cfg in active:
            dup = [s for s, c in active if c["prefix"] == cfg["prefix"] and s != site]
            if dup:
                print(f"[check] 前缀冲突: {site} 与 {dup} 都用 {cfg['prefix']}")
            if cfg["prefix"] in RESERVED_PREFIXES:
                print(f"[check] 前缀保留: {cfg['prefix']} 已占用")
        print(f"[check] {len(active)} active, OK")
        return
    gen_all(sources)
    # cargo fmt 收尾
    subprocess.run(["cargo", "fmt", "--manifest-path", "src-tauri/Cargo.toml"],
                   cwd=ROOT, check=False)


if __name__ == "__main__":
    main()
