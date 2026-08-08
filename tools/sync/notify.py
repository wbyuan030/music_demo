"""SMTP 邮件通知（纯标准库，零依赖）。

用法：
    python tools/sync/notify.py --subject "..." --body-file .sync/report.md
    python tools/sync/notify.py --subject "..." --body "文本"

配置（tools/sync/.env 或环境变量）：
    SMTP_HOST      SMTP 服务器，如 smtp.qq.com / smtp.gmail.com
    SMTP_PORT      默认 465（SSL）
    SMTP_USER      发件账号（通常=登录名）
    SMTP_PASS      授权码/应用密码（不是登录密码）
    SMTP_TO        收件人，逗号分隔

agent.py 集成：各阶段调用 `notify("主题", "正文")`，未配置 SMTP 时静默跳过。
"""
from __future__ import annotations

import argparse
import json
import smtplib
import sys
import urllib.request
import urllib.error
from email.header import Header
from email.mime.text import MIMEText
from email.utils import formataddr
from pathlib import Path

from llm import load_env

SYNC_DIR = Path(__file__).resolve().parent


def _cfg(name: str) -> str:
    return __import__("os").environ.get(name, "")


def send_webhook(subject: str, body: str) -> bool:
    """通用 webhook 通道：POST JSON {subject, body} 到 NOTIFY_WEBHOOK_URL。
    兼容 ntfy.sh / 企业微信 / 飞书 / 钉钉 / Server酱 / Telegram bot 等。"""
    url = _cfg("NOTIFY_WEBHOOK_URL")
    if not url:
        return False
    payload = json.dumps({"subject": subject, "body": body}).encode("utf-8")
    req = urllib.request.Request(
        url, data=payload,
        headers={"Content-Type": "application/json",
                 "User-Agent": "music-demo-sync"},
        method="POST")
    # 本地/内网地址 bypass 系统代理（macOS 常配 127.0.0.1 代理，会拦截本地请求）
    host = url.split("/")[2].split(":")[0]
    if host in ("127.0.0.1", "localhost", "::1"):
        opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
    else:
        opener = urllib.request.build_opener()
    try:
        with opener.open(req, timeout=30) as resp:
            resp.read()
        print(f"[notify] webhook sent ({len(payload)} bytes)", flush=True)
        return True
    except Exception as e:
        print(f"[notify] webhook FAILED: {e}", flush=True)
        return False


def send_email(subject: str, body: str) -> bool:
    """发送邮件。未配置 SMTP 时返回 False 并提示。"""
    host = _cfg("SMTP_HOST")
    user = _cfg("SMTP_USER")
    pw = _cfg("SMTP_PASS")
    to_raw = _cfg("SMTP_TO")
    if not (host and user and pw and to_raw):
        print("[notify] SMTP 未配置（SMTP_HOST/USER/PASS/TO），跳过通知", flush=True)
        return False

    port = int(_cfg("SMTP_PORT") or "465")
    to_list = [t.strip() for t in to_raw.split(",") if t.strip()]

    msg = MIMEText(body, "plain", "utf-8")
    msg["Subject"] = Header(subject, "utf-8")
    msg["From"] = formataddr((str(Header("music-demo sync", "utf-8")), user))
    msg["To"] = ", ".join(to_list)

    try:
        if port == 465:
            server = smtplib.SMTP_SSL(host, port, timeout=30)
        else:
            server = smtplib.SMTP(host, port, timeout=30)
            server.starttls()
        server.login(user, pw)
        server.sendmail(user, to_list, msg.as_string())
        server.quit()
        print(f"[notify] sent to {to_list}", flush=True)
        return True
    except Exception as e:
        print(f"[notify] FAILED: {e}", flush=True)
        return False


def notify(subject: str, body: str) -> None:
    """agent.py 集成入口：webhook 优先，SMTP 兜底；未配置时静默跳过。"""
    load_env(str(SYNC_DIR / ".env"))
    try:
        if send_webhook(subject, body):
            return
        send_email(subject, body)
    except Exception as e:
        print(f"[notify] error: {e}", flush=True)


def main() -> None:
    p = argparse.ArgumentParser(prog="notify.py")
    p.add_argument("--subject", required=True)
    body = p.add_mutually_exclusive_group(required=True)
    body.add_argument("--body", help="正文文本")
    body.add_argument("--body-file", help="从文件读正文")
    args = p.parse_args()
    load_env(str(SYNC_DIR / ".env"))
    content = args.body
    if args.body_file:
        content = Path(args.body_file).read_text(encoding="utf-8")
    ok = send_webhook(args.subject, content)
    if not ok:
        ok = send_email(args.subject, content)
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
