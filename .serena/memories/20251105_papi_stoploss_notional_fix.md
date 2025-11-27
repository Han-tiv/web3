## 2025-11-05 Binance PAPI 止损与名义金额修复
- `apps/rust-trading-bot/src/binance_client.rs`：STOP_MARKET 订单补齐 `closePosition=false` 并移除 `timeInForce`，确保统一账户下止损下单成功；同时把市价单名义金额兜底提升到 21U，低于阈值时自动放大数量并输出提示日志。
- `apps/rust-trading-bot/src/exchange_trait.rs`：`ExchangeClient::get_position` 在筛选前记录总持仓、缺失持仓时发出 warn，便于调试 PAPI map 响应。
- `apps/rust-trading-bot/src/bin/integrated_ai_trader.rs`：持仓监控与清理处的失败日志加入详细错误输出，便于交叉验证 PAPI/FAPI 返回。
- 编译测试时 `cargo test` 会因 `src/bin/fund_monitor.rs` 的 Chat 解构错误 (E0308) 中断，需后续跟进。