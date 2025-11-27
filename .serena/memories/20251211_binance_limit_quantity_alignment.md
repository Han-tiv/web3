## 2025-12-11
- BinanceClient::limit_order 现在在格式化数量前按 step_size 对齐数量并确保不少于 min_qty，与 market_order 的数量校验保持一致，避免因数量精度导致订单被拒。