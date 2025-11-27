## Binance PAPI Map 兼容处理
- `apps/rust-trading-bot/src/binance_client.rs` 中 `ExchangeClient::get_klines` 先获取文本响应，依次尝试解析为数组或 `{symbol: [...]}` map，无法解析时输出截断预览。
- `get_positions` 在 PAPI 请求成功后增加 info 级别预览日志，辅助分析返回格式；维持数组与 map 的双路径解析。
- 统一提示：Binance PAPI 部分接口可能返回 map 格式，后续扩展接口时优先考虑双格式解析策略。