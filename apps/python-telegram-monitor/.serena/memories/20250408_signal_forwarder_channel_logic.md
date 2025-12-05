## 方程式频道信号处理
- 2025-04-08 在 `signal_forwarder.py` 的 `handle_message` 中加入频道分支：
  - `@BWE_OI_Price_monitor` 走 `parse_fangchengshi_signal`/`format_fangchengshi_signal`，失败直接跳过并记录日志。
  - 其它频道保留原先 Valuescan 风险过滤+币种提取流程。
- 两个分支都要设置 `symbol` 与 `raw_message` 供后续 `signal_data` 使用，`raw_message` 不再固定等于原始文本。