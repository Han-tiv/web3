# build_entry_analysis_prompt 更新
- 在 apps/rust-trading-bot/src/deepseek_client.rs 的开仓分析提示词 JSON 输出格式中, 强制 stop_loss/take_profit 必须为具体数字且不可为 null。
- 新增说明要求止损/止盈基于K线形态识别的关键支撑阻力位, 禁止仅用固定百分比, 并在重要说明段落补充细则。