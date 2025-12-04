## 2025-03-21 Binance 杠杆实时获取
- 在 `apps/rust-trading-bot/src/binance_client.rs` 增加了 `get_symbol_leverage`，调用 `/fapi/v2/positionRisk?symbol=` 返回交易对当前杠杆，并在值异常时抛错。
- `apps/rust-trading-bot/src/bin/integrated_ai_trader/core/entry_manager.rs` 现在会在风险计算前调用该方法，日志输出实际杠杆，失败时降级为 10x。
- 后续若要缓存杠杆，可在 EntryManager 层用 `Arc<RwLock<HashMap<String,f64>>>` 记录最近值以减少 API 调用。