# deepseek_client轻微亏损止损策略
- 日期：2025-12-09
- 文件：apps/rust-trading-bot/src/deepseek_client.rs
- 更新内容：轻微亏损（-0.5% ~ -1.5%）由FULL_CLOSE调整为多条件PARTIAL_CLOSE（30-50%减仓），新增减仓后观察提示。
- 注意：中度与严重亏损段落保持FULL_CLOSE逻辑，不可误改。