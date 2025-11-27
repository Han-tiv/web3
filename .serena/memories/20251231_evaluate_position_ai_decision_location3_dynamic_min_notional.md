# evaluate_position_with_ai Location3 动态最小平仓金额
- evaluate_position_with_ai() 中 AI 决策的 PARTIAL_CLOSE 分支已删除常量 MIN_NOTIONAL，改为调用交易所规则并结合最新标记价格计算部分平仓金额。
- 当建议金额低于 min_notional 时会动态放大平仓比例，保留相同日志格式；若持仓价值不足则直接触发 FULL_CLOSE。
- 返回结构保持为 Some(PositionAction::PartialClose|FullClose)，调用方无需调整。