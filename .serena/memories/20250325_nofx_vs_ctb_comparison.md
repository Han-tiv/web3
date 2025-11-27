# NOFX vs crypto-trading-bot 对比结论（2025-03-25）

- 目的：在 rust-trading-bot 已为主项目（8.88 分）的前提下，评估 NOFX 与 crypto-trading-bot 哪个更值得保留为 Go 侧参考实现。

- 总体结论：
  - NOFX = 产品级 Agentic Trading OS（Gin + React + 多交易所 + 用户/JWT/2FA + Telegram），功能最全，但 Go 核心交易逻辑集中在 auto_trader/decision 等大文件，数据库偏配置，不存完整交易事实。
  - crypto-trading-bot = CloudWeGo Eino Graph 多智能体交易流水线（market/crypto/sentiment/position_info/trader），配合 TradeCoordinator + StopLossManager + SQLite(trading_sessions/positions/stoploss_events/balance_history)，架构更清晰，适合在 Rust 中一一重构。

- 关键差异：
  - 策略/风控：NOFX 在代码侧硬约束风险回报≥3:1、单币仓位上限、最小名义价值、止损/止盈关系与回撤平仓；CTB 将资金使用率 30/50/70 档、置信度和 RR 规则写入 Prompt，并通过 TradingDecision + calculatePositionSize + 服务器端 STOP_MARKET + 对账实现较完整链路。
  - 数据：NOFX 的 SQLite 主要管理 users/ai_models/exchanges/traders/system_config/audit_logs；CTB 的 SQLite 直接建模交易会话、持仓、止损事件与余额历史，更利于统计和监控。

- 推荐：
  - 若只能保留一个 Go 项目作为长期参考，建议保留 **apps/crypto-trading-bot**；
  - NOFX 主要保留文档与若干关键设计（多交易所抽象、JWT/2FA、Telegram 告警）作为 rust-trading-bot 的产品与运维层参考。