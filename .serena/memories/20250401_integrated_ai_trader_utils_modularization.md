# Integrated AI Trader 工具模块重构
- integrated_ai_trader 现在使用 `utils/` 目录，包含 `validators`, `converters`, `calculators` 子模块。
- `is_meme_coin`, `timestamp_ms_to_datetime`, `normalize_signal_type`, `map_confidence_to_score` 从 `trader.rs` 提取到对应模块，`calculators` 预留给波动率计算。
- `trader.rs` 通过 `super::utils` 引入上述函数，并复用 `modules::config::MEME_COINS` 常量。
- Cargo fmt 应当在 `apps/rust-trading-bot` 目录运行，否则会因缺少 `Cargo.toml` 报错。