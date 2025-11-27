## 2025-02-19
- `BinanceClient::open_long/open_short` 统一改为调用 `limit_order`，使用当前价乘以 1.001/0.999 并传入 positionSide，避免市价单 taker 费用。
- `ExchangeClient::get_positions` 记录 PAPI 返回体（截取 5000 字符）并新增包装格式解析路径，同时在解析成功时输出持仓条数。