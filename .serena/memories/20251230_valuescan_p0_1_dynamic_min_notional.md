# Valuescan P0-1 最小平仓金额逻辑
- integrated_ai_trader.rs 中 Valuescan P0-1 规则已改为动态读取交易对 min_notional 并使用最新标记价格计算部分平仓金额。
- 当 AI 建议的部分平仓金额不足时，会根据当前持仓总价值计算最小满足比例，若比例<=100%则放大平仓比例，否则执行全部平仓。
- 警告日志格式与 build_action_from_decision 中的“智能部分平仓比率调整”保持一致，便于统一排查。