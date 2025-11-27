# Gemini Client Prompt 约束更新
- `build_entry_analysis_prompt` 在 "AI综合决策原则" 与 "输出格式" 之间增加 "AI入场价格约束" 段落。
- 约束要求 entry_price 优先落在量化入场区，若突破最多偏离上界+15%，并在 reason 中说明；当偏离超过20% 时建议 SKIP，除非存在明确反转证据。
- 段落保留 1h主入场区/15m辅助区/量化推荐占位符，强化 AI 给价与量化区间的一致性。