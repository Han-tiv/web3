# 延迟开仓队列清理
- IntegratedAITrader::execute_ai_trial_entry 在成功提交试探建仓订单后，会立刻从 pending_entries 中移除对应 symbol，避免重复重试。