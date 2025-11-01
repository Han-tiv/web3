# nof1.ai Prompt 分析（2025-10-27 02:00 UTC 快照）

## 数据来源
- 通过 `curl https://nof1.ai/api/conversations` 获取最近 100 条会话记录。
- 每条记录含 `model_id`, `user_prompt`, `llm_response`, `timestamp` 等字段；`user_prompt` 即平台喂给模型的完整提示词。
- 本文摘录自 `/tmp/nof1-api-conversations.json`（未纳入仓库）并按模型分组分析。

## 通用提示模板
1. **开场元数据**：固定句式（“It has been X minutes… you've been invoked Y times…”），说明模型运行时长与调用次数。
2. **数据说明**：强调所有行情序列按“最早 → 最新”排序，默认 3 分钟时间粒度。
3. **市场板块**：依次覆盖 BTC、ETH、SOL、BNB、XRP、DOGE，每个板块包含：
   - `current_price`、EMA、MACD、RSI 当前值
   - 最新永续合约 open interest / funding rate
   - 10 条 3 分钟级别价量/指标数组
   - 4 小时级别 EMA / ATR / Volume / MACD / RSI 摘要
4. **账户段落**：列出 `Current Total Return`、`Available Cash`、`Current Account Value`、持仓列表（含杠杆、止盈/止损、信心、风险敞口、订单 ID）。
5. **收尾指标**：`Sharpe Ratio` 当前值。

模板在 6 个模型之间完全一致，只有数值（时间戳、指标、仓位数据等）会随实时状态更新。

## 各模型快照
> 注：以下仅展示 `user_prompt` 的开头 4 行与账户价值，完整内容可通过 API 获取。

### gpt-5
```
It has been 6535 minutes since you started trading. The current time is 2025-10-27 01:58:21.712765 and you've been invoked 2626 times.
...
**Current Account Value:** 4156.7
Sharpe Ratio: -0.688
```

### claude-sonnet-4-5
```
It has been 6525 minutes since you started trading. The current time is 2025-10-27 01:57:49.385972 and you've been invoked 3083 times.
...
**Current Account Value:** 4137.18
Sharpe Ratio: -0.712
```

### grok-4
```
It has been 6531 minutes since you started trading. The current time is 2025-10-27 01:57:08.668783 and you've been invoked 2647 times.
...
**Current Account Value:** 4179.02
Sharpe Ratio: -0.681
```

### gemini-2.5-pro
```
It has been 6529 minutes since you started trading. The current time is 2025-10-27 01:59:24.338720 and you've been invoked 3142 times.
...
**Current Account Value:** 4163.52
Sharpe Ratio: -0.689
```

### deepseek-chat-v3.1
```
It has been 6531 minutes since you started trading. The current time is 2025-10-27 01:59:39.567732 and you've been invoked 2592 times.
...
**Current Account Value:** 4149.68
Sharpe Ratio: -0.695
```

### qwen3-max
```
It has been 6528 minutes since you started trading. The current time is 2025-10-27 01:59:35.410133 and you've been invoked 4061 times.
...
**Current Account Value:** 4142.91
Sharpe Ratio: -0.701
```

## 结论
- 所有模型共用相同 prompt 骨架，平台通过实时行情与仓位数据填充动态字段，无模型定制策略或语气差异。
- prompt 体量约 9k~11k 字符，主要是市场数据，未包含额外 “规则/限制” 小节。
- 若需复现，可直接请求 `/api/conversations` 并根据 `model_id` 过滤；接口当前未要求鉴权。

## 推测的系统级提示（不可见但可由行为反推）
根据 `/api/conversations` 暴露的 `cot_trace` 与模型输出，可以推断平台在模型会话开始时注入了额外指令，核心要点如下：

1. **角色定位**：多次出现“我在 Hyperliquid 上作为系统化交易员”“swing_trading” 等字样，表明系统 prompt 明确要求模型扮演 Hyperliquid 的波段交易员。
2. **决策范围**：`cot_trace` 反复提到“只能 hold 或 close”“不得开新仓或加仓”“只有在 invalidation 条件触发时才可提前平仓”，说明初始指令限制模型的动作集合。
3. **风险与规则**：模型会自检杠杆、风险敞口、止盈止损，与 `cot_trace` 中的“必须遵循既有 exit plan”“stop-loss 由交易所托管”相呼应。
4. **输出格式**：尽管 `user_prompt` 未提到格式要求，`cot_trace` 明确“他们要一个单一 JSON 对象”“无需额外 justification”，实际 `llm_response` 也始终为 `{symbol: {...}}` 结构，推断系统 prompt 规定了精确的 JSON 架构（字段包括 signal、quantity、stop_loss、profit_target、invalidation_condition、confidence、leverage、risk_usd 等）。
5. **节奏要求**：文中提及“数据每 3 分钟推送，需要快速但谨慎决策”，说明初始提示强调节奏与避免过度交易。
6. **Schema 示例**：多条 `cot_trace` 直接给出占位模板：
   ```
   "COIN": {
     "trade_signal_args": {
       "coin": "COIN",
       "signal": "hold" | "close_position",
       "quantity": <full current size>,
       "profit_target": <float>,
       "stop_loss": <float>,
       "invalidation_condition": "<string>",
       "leverage": <int 5–40>,
       "confidence": <0–1>,
       "risk_usd": <float>,
       "justification": "<简短原因，仅在 close 时需要>"
     }
   }
   ```
   该模板在 deepseek、gemini 等模型的思维链中反复出现，证明系统 prompt 要求模型最终返回此 JSON 结构。

由于这些约束未出现在 `user_prompt` 中，却在模型思考与输出中持续出现，可合理断定它们来源于不可见的系统初始提示。精确措辞无法直接获取，但以上规则应构成其核心内容。
