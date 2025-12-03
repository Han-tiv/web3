# Integrated AI Trader - Utils 模块（Phase 1）
- 新增 `utils::{validators, calculators, converters}` 三个子模块，`mod.rs` 统一 `pub use`。
- `validators::validate_entry_zone` 与 `is_meme_coin` 拆自 `trader.rs`，逻辑保持一致，前者已参数化为纯函数。
- `calculators::calculate_volatility` 现接受泛型 `ExchangeClient` 引用和 `volatility_cache`（`Arc<RwLock<_>>`），负责缓存与超时处理。
- `converters::{timestamp_ms_to_datetime, normalize_signal_type, map_confidence_to_score}` 提供通用时间/信号转换函数，供 trader 与前端复用。
- Trader 模块暂时仍保留原方法，后续计划通过委托调用新模块函数。