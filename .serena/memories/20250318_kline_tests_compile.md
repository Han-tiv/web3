# Kline 单元测试修复与示例更新
- `deepseek_client::Kline` 现已 `derive(Default)`，测试可用 `..Default::default()` 补齐新增字段，避免结构变更再次导致编译失败。
- `key_level_finder.rs`、`smart_money_tracker.rs`、`entry_zone_analyzer.rs`、`launch_signal_detector.rs` 的测试构造器统一复用辅助函数，减少重复字段填写。
- `exchange_trait::ExchangeClient::adjust_position` 的文档示例包裹在 `async fn` 并使用 `no_run`，确保 `cargo test` 不再因 doctest 报错。
