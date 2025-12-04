# OI 异动优先候选币种逻辑
- `trader/auto_trader.go#getCandidateCoins` 现逻辑：本地OI监控器>自定义币种>数据库默认>AI500+OI Top fallback。
- 使用 `pool.GetOITopPositions()` 过滤变化率>8%的币种，Sources标记为`oi_spike`，并用 `usedSymbols` 去重。
- 自定义币种保留 `custom` 来源；默认币种在无候选或作为尾部补充时标记 `default`/`default_backup`，确保最终列表包含基础币种。