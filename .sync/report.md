# Upstream sync report

- upstream:  (5d6b8c8cd19785c3086ae3a9ec618c45e25eb3bc)
- commits analyzed: 50
- T1 纯数据: 1 | T2 逻辑: 1 | T3 新来源: 0 | T4 无关: 48

## 需人工确认

- 5d5b634d [ie/youtube] Add `web_embedded` client fallbacks (#17462): 影响 youtube/api.rs 中 INNERTUBE_CLIENTS 默认客户端配置（_DEFAULT_AUTHED_CLIENTS、_DEFAULT_PREMIUM_CLIENTS），以及 fallback 逻辑（age-restricted videos 追加 web_embedded）

## 测试

- cargo test: 未全绿（见 .sync/test-output.txt），需人工修复
