## 2025-11-05 build_enhanced_prompt重构
- 调整integrated_ai_trader的Prompt结构，Valuescan频道信号作为核心，1h关键位与短周期指标作为辅助
- 删除全部24H涨跌/高低引用，新增交易核心原则、入场条件、止盈止损等中文分节
- 将format_entry_condition用于展示关键位动态判断，保持JSON输出要求不变