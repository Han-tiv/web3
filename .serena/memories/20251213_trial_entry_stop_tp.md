# 2025-12-13 Trial Entry Risk Controls
- DeepSeek prompt now要求AI输出take_profit并在reason中解释止盈/止损逻辑。
- integrated_ai_trader在试探建仓订单成交后立即调用exchange设置止损与限价止盈，并把返回的order_id记录到PositionTracker，便于后续监控。