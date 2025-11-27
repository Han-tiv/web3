# AI 覆盖入场区容差
- `execute_ai_trial_entry` 现接收 `is_ai_override` 参数，由 WaitForPullback 分支在 AI HIGH 覆盖时传入 true。
- `validate_entry_zone` 基于该参数应用不同容差：正常 ±3%，AI 覆盖 ±10%，并在 AI 覆盖突破标准区间但落在扩展区间内时输出详细日志。
- 目的：避免 AI 高信心信号因原先严格的入场区验证被拒绝，同时保持一般场景的风控。