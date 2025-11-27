# 集成AI持仓提示新增保护单价格
- `BinanceClient::OrderStatus` 现包含 `price` 以及可选 `stop_price` 字段, `get_order_status` 会解析 `price`/`stopPrice` 字段。
- `evaluate_position_with_ai` 在生成 prompt 前会实时查询当前止损/止盈挂单价格, 并把结果传给 `build_position_management_prompt`。
- `DeepseekClient::build_position_management_prompt` 在【持仓信息】段展示当前保护单价格, 让 AI 评估时能参考真实生效的止损/止盈。