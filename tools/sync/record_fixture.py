"""T3 新来源 fixture 录制工具。

用法（dev 时，协议实现前先录真实响应）：
    python tools/sync/record_fixture.py --name search --out src-tauri/src/extractor/showroom/fixtures/search.json \
        --url "https://www.showroom-live.com/api/..."
    python tools/sync/record_fixture.py --from-ytdlp "scsearch5:keyword" \
        --out src-tauri/src/extractor/soundcloud/fixtures/search.json

录制完成后提交 fixture JSON，实现者按 gen_adapter 文档中的模板写
fixture 解析测试（离线、CI 必跑）。
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent


def record_http(url: str, out: Path, headers: dict[str, str] | None = None) -> None:
    h = dict(headers or {})
    h.setdefault("User-Agent",
                 "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
    req = urllib.request.Request(url, headers=h)
    with urllib.request.urlopen(req, timeout=30) as resp:
        body = resp.read()
    # 尽力归一化：JSON 则 pretty-print，非 JSON 存原文
    try:
        data = json.loads(body)
        out.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n")
    except json.JSONDecodeError:
        out.write_bytes(body)
    print(f"[record] {out} ({out.stat().st_size} bytes)")


def record_ytdlp(query: str, out: Path, ytdlp: str = "../yt-dlp") -> None:
    """用 yt-dlp -J 录制（dev 参考：上游解析结果可当 oracle fixture）。"""
    r = subprocess.run(
        [sys.executable, "-m", "yt_dlp", "-J", "--no-warnings", "--skip-download", query],
        cwd=ytdlp, capture_output=True, text=True, timeout=60)
    if r.returncode != 0:
        sys.exit(f"yt-dlp failed: {r.stderr[-500:]}")
    data = json.loads(r.stdout)
    out.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n")
    print(f"[record] {out} from yt-dlp ({out.stat().st_size} bytes)")


def main() -> None:
    p = argparse.ArgumentParser(prog="record_fixture.py")
    src = p.add_mutually_exclusive_group(required=True)
    src.add_argument("--url", help="真实 API URL（HTTP GET）")
    src.add_argument("--from-ytdlp", help="yt-dlp 查询串（如 scsearch5:xxx 或视频 URL）")
    p.add_argument("--out", required=True, help="输出路径，如 src-tauri/src/extractor/<site>/fixtures/search.json")
    p.add_argument("--header", action="append", default=[], help="额外 header，可多次（Key: Value）")
    args = p.parse_args()

    out = (ROOT / args.out).resolve()
    if not out.is_relative_to(ROOT):
        sys.exit("out 必须在仓库内")
    out.parent.mkdir(parents=True, exist_ok=True)

    if args.url:
        headers = {}
        for h in args.header:
            k, _, v = h.partition(":")
            headers[k.strip()] = v.strip()
        record_http(args.url, out, headers)
    else:
        record_ytdlp(args.from_ytdlp, out)


if __name__ == "__main__":
    main()
