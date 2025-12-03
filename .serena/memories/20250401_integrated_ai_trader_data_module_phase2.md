# integrated_ai_trader data 模块
- Phase 2 将 trader.rs 中的 position_trackers、波动率缓存与交易历史写入逻辑拆分为 data 模块。
- `TrackerManager` 使用 Arc<RwLock<HashMap<String, PositionTracker>>> 封装同步、孤儿清理、读写接口。
- `CacheManager` 负责 `VolatilityCacheEntry` 的写入、读取与过期清理，复用 `VOLATILITY_CACHE_TTL_SECS` 配置。
- `HistoryRecorder` 以 Arc<Database> 注入，`record_trade` 接收 `TradeRecordParams`，可由 tracker/staged snapshot 自动推导入场信息。
- data/mod.rs 统一导出上述结构，供未来模块化访问。