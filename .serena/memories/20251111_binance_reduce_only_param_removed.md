## 2025-11-11
- BinanceClient::limit_order 不再往查询字符串追加 reduceOnly 参数；Binance API 仅依赖 positionSide 判定平仓。
- 函数签名仍保留 bool 参数但重命名为 `_reduce_only` 以兼容现有调用，未来可以逐步移除。
- 需要关注其它使用硬编码 `reduceOnly=true` 的下单函数（如 set_limit_take_profit），若再次触发 -1106，可参考本次改动。