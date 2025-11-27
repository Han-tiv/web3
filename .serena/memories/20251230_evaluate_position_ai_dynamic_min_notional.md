# Evaluate Position AI 最小平仓金额
- evaluate_position_with_ai() 中 AI 决策的 PARTIAL_CLOSE 分支不再使用硬编码 $20 MIN_NOTIONAL，改为读取交易对规则并使用最新标记价格计算部分平仓金额。
- 当建议金额不足时会自动放大平仓比例或触发全平仓，日志格式与 build_action_from_decision 的“智能部分平仓比率调整”完全一致。