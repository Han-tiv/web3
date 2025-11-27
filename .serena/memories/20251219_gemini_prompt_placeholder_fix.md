# build_entry_analysis_prompt 参数/占位符约束
- Gemini Client 的 `build_entry_analysis_prompt` 之前在 "AI入场价格约束" 段落中为 1h/15m 入场区和量化推荐价格提供了 4 个占位符。
- 这些信息已在同一个 prompt 的 "量化入场区参考" 段落给出，因此再次保留占位符会导致 format! 需要额外参数。
- 2025-12-19 起，这四个占位符被文本说明替代，强调“上文已提供”，当前 format! 只需 12 个参数；若新增字段请同步更新参数列表。