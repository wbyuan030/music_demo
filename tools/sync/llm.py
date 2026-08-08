"""OpenAI 兼容 LLM 客户端（纯标准库，零第三方依赖）。

env:
    LLM_BASE_URL  API 端点，如 https://api.openai.com/v1 或 https://openrouter.ai/api/v1
    LLM_API_KEY   密钥
    LLM_MODEL     模型名，如 gpt-4o-mini / deepseek-chat / gemini-2.0-flash
    LLM_TIMEOUT   请求超时秒数（默认 180）
    LLM_MAX_RETRIES  429/网络错误重试次数（默认 3）
"""
from __future__ import annotations

import json
import os
import time
import urllib.error
import urllib.request
from typing import Any


def _env(name: str, default: str = "") -> str:
    return os.environ.get(name, default)


def _post(body: dict[str, Any], timeout_s: int) -> dict[str, Any]:
    """POST /chat/completions，带 429/网络重试。"""
    base = _env("LLM_BASE_URL", "https://api.openai.com/v1").rstrip("/")
    api_key = _env("LLM_API_KEY")
    if not api_key:
        raise RuntimeError("LLM_API_KEY 未设置（env 或 .env）")
    body["model"] = _env("LLM_MODEL", "gpt-4o-mini")
    req = urllib.request.Request(
        f"{base}/chat/completions",
        data=json.dumps(body).encode("utf-8"),
        headers={
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
        },
        method="POST",
    )
    max_retries = int(_env("LLM_MAX_RETRIES", "3"))
    last_err: Exception | None = None
    for attempt in range(max_retries):
        try:
            with urllib.request.urlopen(req, timeout=timeout_s) as resp:
                return json.loads(resp.read().decode("utf-8"))
        except urllib.error.HTTPError as e:
            if e.code == 429 and attempt < max_retries - 1:
                wait = 5 * (attempt + 1)
                print(f"[llm] 429 rate limited, retry in {wait}s", flush=True)
                time.sleep(wait)
                continue
            raise RuntimeError(f"LLM HTTP {e.code}: {e.read()[:500]!r}") from e
        except (urllib.error.URLError, TimeoutError, json.JSONDecodeError) as e:
            last_err = e
            if attempt < max_retries - 1:
                wait = 5 * (attempt + 1)
                print(f"[llm] transient error {e!r}, retry in {wait}s", flush=True)
                time.sleep(wait)
                continue
    raise RuntimeError(f"LLM 调用失败（重试耗尽）: {last_err!r}")


def chat(
    messages: list[dict[str, str]],
    *,
    json_mode: bool = False,
    max_tokens: int = 4096,
    temperature: float = 0.2,
    timeout: int | None = None,
) -> Any:
    """调用 chat completions。json_mode=True 时返回解析后的 JSON 对象。"""
    body: dict[str, Any] = {
        "messages": messages,
        "temperature": temperature,
        "max_tokens": max_tokens,
    }
    if json_mode:
        body["response_format"] = {"type": "json_object"}
    timeout_s = timeout or int(_env("LLM_TIMEOUT", "180"))
    payload = _post(body, timeout_s)
    content = payload["choices"][0]["message"]["content"]
    if json_mode:
        return json.loads(content)
    return content


def chat_with_tools(
    messages: list[dict[str, Any]],
    tools: list[dict[str, Any]],
    *,
    max_tokens: int = 4096,
    temperature: float = 0.2,
    timeout: int | None = None,
) -> tuple[list[dict[str, Any]], str | None]:
    """带 function calling 的 chat。

    返回 (final_messages, final_content)：调用方执行工具后把结果作为
    tool 角色消息追加进 final_messages 再调，直到响应的 content 非空。
    """
    body: dict[str, Any] = {
        "messages": messages,
        "tools": tools,
        "temperature": temperature,
        "max_tokens": max_tokens,
    }
    timeout_s = timeout or int(_env("LLM_TIMEOUT", "180"))
    payload = _post(body, timeout_s)
    msg = payload["choices"][0]["message"]
    messages = messages + [msg]
    return messages, msg.get("content")


def load_env(path: str = ".env") -> None:
    """简单 .env 加载（KEY=VALUE 行，忽略注释/空行），不覆盖已有 env。"""
    if not os.path.exists(path):
        return
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#") or "=" not in line:
                continue
            k, _, v = line.partition("=")
            k, v = k.strip(), v.strip().strip("\"'")
            if k and k not in os.environ:
                os.environ[k] = v
