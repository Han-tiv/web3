# Integrated AI Trader 部分平仓止损数量修复
- 文件：`apps/rust-trading-bot/src/bin/integrated_ai_trader.rs`
- 原逻辑用 `total_quantity - executed_qty` 重新计算剩余仓位，导致当交易所返回异常成交量时会把剩余仓位算成 0。
- 新实现改为：`actual_remaining = remaining_quantity + max(close_quantity - executed_qty, 0)`，确保至少保留计划剩余，并把未成交的部分继续计入止损数量。
- 影响：部分平仓失败或部分成交时，将正确更新止损单数量，而不是误删持仓。