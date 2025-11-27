## 2025-12-12 Integrated AI Alpha/FOMO 过滤
- 位置：apps/rust-trading-bot/src/bin/integrated_ai_trader.rs
- 调整：在原有 Alpha 关键词判定上新增 FOMO 关键词集（暴涨/拉升/突破/异动/急拉/爆发/fomo），并以 `is_special_coin = is_alpha || is_fomo` 统一用于价格阈值与交易过滤。
- 日志：Skip 日志更新为“⏭️ 跳过非Alpha/FOMO币种…”，价格相关日志保持 Alpha 文案。
- 注意：若未来区分 Alpha/FOMO 日志或阈值，需要同时调整 `is_special_coin` 分支与价格提示文案。