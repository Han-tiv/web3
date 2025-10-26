# ValueScan 接口集成指引

> 日期：2025-10-17  执行者：Codex

## 1. 凭证配置

- 环境变量：`VALUESCAN_BEARER_TOKEN`、`VALUESCAN_ACCESS_TICKET`（见仓库根目录 `.env:90-92`）。
- 使用 axios 请求时，将两个头部分别写入 `Authorization: Bearer ...` 与 `Access-Ticket: ...`。
- 提醒：凭证敏感，避免在日志中打印；建议定期轮换并设置访问控制。

## 2. 已确认可用的接口

| 接口 | 方法 | 主要字段 | 说明 |
| --- | --- | --- | --- |
| `/api/chance/getFundsMovementPage` | POST | `symbol`, `tradeType`, `beginTime`, `endTime`, `price`, `beginPrice`, `number24h`, `numberNot24h`, `gains`, `decline`, `percentChange24h`, `marketCap`, `alpha`, `fomo`, `observe`, `favor` | 资金异动榜，支持分页与排序；`number24h`/`numberNot24h` 对应小/大周期异动次数；`alpha`/`fomo` 为标签。 |
| `/api/chance/getChangeCoinPage` | POST | `symbol`, `price`, `date`, `oldPrice`, `score`, `scoreChange`, `grade`, `gains`, `decline`, `marketCap`, `circulationRate`, `percentChangeRanking`, `mockTrade`, `marketCapRanking` | 风险/动量评分榜，`grade` (1≈I, 2≈II) 表示风险等级，`percentChangeRanking` 提供多周期涨跌排名。 |
| `/api/account/message/getWarnMessage` | GET | 外层 `id`, `title`, `messageType`, `tradeType`, `createTime`, `content`；`content` 字符串需二次 JSON.parse，字段包含 `fundsMovementType`, `symbol`, `price`, `percentChange24h`, `updateTime`, `icon`, `observe` 等 | 官方告警流（如资金异常），可作为风控提醒。 |

相关资源对照表：

- 用户提供的 UI 字段与 JSON 映射：
  - `代币名称` ↔ `symbol`
  - `首次/最新异动时间` ↔ `beginTime` / `endTime`（毫秒时间戳）
  - `当前币价` ↔ `price`
  - `标记价格` ↔ `beginPrice` 或 `oldPrice`
  - `小周期异动/大周期异动` ↔ `number24h` / `numberNot24h`
  - `AI 评分` ↔ `score`，`风险等级` ↔ `grade`
  - `最大涨幅/最大跌幅` ↔ `gains` / `decline`
  - `市值` ↔ `marketCap`
  - `流通率` ↔ `circulationRate`

## 3. social-monitor 集成建议

- **请求封装**：在 `apps/social-monitor/services/aggregator/src/index.ts` 中定义 `ValueScanClient`，以 axios 封装上述接口，请求时即时处理，无需 cron 调度。
- **同步端点**：新增 REST 路由（如 `/api/valuescan/funds`、`/api/valuescan/risk`、`/api/valuescan/alerts`），在处理器内:
  1. 调用对应 ValueScan 接口；
  2. 解析响应，转换数值类型并返回；
  3. 可选：将结果通过现有 WebSocket 推送给前端订阅者。
- **速率控制**：若短时间有多次请求，可在服务内做简单缓存（例如存储上次响应 + TTL）或对外暴露 `cache=true` 参数，避免触发外部限流。
- **数据落地**：如需后续分析，可把解析后的数据写入 Redis / 数据库，键结构建议包含 `valuescan:funds:{symbol}`、`valuescan:risk:{symbol}`、`valuescan:alerts:{id}`。

## 4. 其他情报型接口概览

根据官网资源列表，以下路径同样提供 AI/监控数据，可按需扩展：

- `chance/*`：历史资金异动、币种追踪配置。
- `signal/*` / `signals/*`：链上信号、精选策略与消息。
- `monitor/message/*`：自定义监控告警配置与读取接口。
- `analysis/coin/*`、`fund/*`、`data/*`：跨交易所资金流、热钱榜、期货/现货主力数据。
- `track/*`：持仓成本、收益曲线与自定义信号订阅。

上述接口多需要相同凭证，具体字段可参考对应 `.ts` 词条或通过 axios 请求验证。

## 5. 注意事项

- 所有返回时间均为毫秒时间戳，转换时注意时区（示例值对应 2025-10-17 23:40 CST）。
- `content` 字段若为字符串需二次 JSON 解析；数值类型默认字符串，使用前请转换。
- 凭证长时间使用风险较高，应建立轮换和失效通知机制。
