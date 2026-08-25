# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 34
- T1 纯数据: 2 | T2 逻辑: 2 | T3 新来源: 0 | T4 无关: 30

## 需人工确认

- cf68b8f4 [ie/youtube:tab] Always extract channel metadata (#17386): youtube/mod.rs 或 youtube/api.rs 中的 tab/playlist 解析逻辑、channel metadata 提取
- 5d5b634d [ie/youtube] Add `web_embedded` client fallbacks (#17462): youtube/api.rs — 默认客户端列表和认证/高级账户回退逻辑（_DEFAULT_AUTHED_CLIENTS / _DEFAULT_PREMIUM_CLIENTS）；可能涉及 age-gated / unplayable 时追加 web_embedded 客户端的路径

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
