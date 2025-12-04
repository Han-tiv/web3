## 2025-03-21 EntryManager 杠杆上限
- `apps/rust-trading-bot/src/bin/integrated_ai_trader/core/entry_manager.rs` 获取实际杠杆后立刻应用 30x 上限保护, 并对日志输出实际杠杆/降级后杠杆的止损距离
- 超出上限时记录 `⚠️  <symbol>: 实际杠杆 xx.x(止损距yy.y%),超过安全上限,强制降级为 30.0x(止损距zz.z%)` 日志, 随后再进行最小 1x 校验