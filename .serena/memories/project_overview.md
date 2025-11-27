# 项目概览
- 名称：Web3 Monorepo，聚焦社交舆情监控与 Rust 量化/复制交易机器人。
- 主要子项目：`apps/rust-trading-bot`（Rust 异步交易系统）、`apps/social-monitor`（多平台情报聚合）等。
- Rust Trading Bot 特性：Tokio 异步、Binance/OKX/Gate/Hyperliquid 多交易所、AI/主力追踪策略、Telegram 控制面板。
- Monorepo 管理：使用 Node.js/npm + Turbo 处理跨项目构建与脚本调度。
- 环境需求：Node.js 18+、Rust 1.70+、Docker 与 Docker Compose、npm 9+。
- 关键文档入口：根目录 `README.md`、`docs/README.md`、`apps/rust-trading-bot/docs`。