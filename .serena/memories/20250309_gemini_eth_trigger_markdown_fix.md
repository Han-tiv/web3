# 2025-03-09 Gemini ETH Analyzer 修复
- 修复 ParsedAction::TriggerOrder 中做空方向触发类型判断，遵循 Binance STOP_MARKET 为向下触发、TAKE_PROFIT_MARKET 为向上触发的规则。
- normalize_action_line 现支持去除 Markdown 粗体、项目符号、序号及冒号后内容，确保 “止盈止损订单撤单” 等指令可正确解析。
- 相关文件：apps/rust-trading-bot/src/bin/gemini_eth_analyzer.rs。