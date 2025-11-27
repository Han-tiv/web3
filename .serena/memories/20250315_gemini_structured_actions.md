## Gemini ETH analyzer 结构化指令
- `apps/rust-trading-bot/src/bin/gemini_eth_analyzer.rs` 现在要求 Gemini 在“最终操作建议”段落给出编号的结构化操作列表。
- 新增 `parse_structured_actions` 直接解析该段落并生成 `ParsedAction`，旧的 `parse_trading_signal` 以 `#[allow(dead_code)]` 形式保留备用。
- `ParsedAction` 增加 `SetTPSL` 分支，可对现有仓位直接设置止盈止损；旧的关键字/价格解析辅助函数迁移为 `_with_patterns`，结构化解析使用简化的 `extract_price`。