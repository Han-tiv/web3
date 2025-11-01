# 📚 Web3 Trading Project - 文档中心

**完整的加密货币交易系统文档库**

---

## 🎯 快速导航

### 🚀 新手入门
- [项目总览](../README.md) - 项目介绍和整体架构
- [快速开始](../apps/rust-trading-bot/docs/user-guide/QUICKSTART.md) - 5分钟上手
- [环境配置](./guides/ENV_CONFIGURATION_GUIDE.md) - 环境变量配置指南

### 📖 核心文档
- [系统架构](./architecture/ARCHITECTURE.md) - 整体架构设计
- [部署指南](./deployment/DEPLOYMENT_GUIDE.md) - 生产环境部署
- [安全分析](./security/SECURITY_ANALYSIS.md) - 安全配置和审计

---

## 📂 文档结构

```
docs/
├── README.md                          ← 本文件 (文档导航中心)
│
├── architecture/                      ← 架构文档
│   └── ARCHITECTURE.md                  系统整体架构
│
├── security/                          ← 安全文档
│   ├── SECURITY_ANALYSIS.md             安全分析报告
│   └── SECURITY_SUMMARY.md              安全总结
│
├── optimization/                      ← 优化文档
│   ├── README.md                        优化报告导航
│   ├── WEB3_PROJECT_OPTIMIZATION.md     项目优化报告
│   ├── OPTIMIZATION_REPORT.md           Phase 1 优化报告
│   ├── PHASE_2_PERFORMANCE_REPORT.md    Phase 2 性能报告
│   ├── PHASE_3_INTELLIGENCE_REPORT.md   Phase 3 智能报告
│   ├── SHORT_TERM_OPTIMIZATION_COMPLETE.md
│   └── PROJECT_REFACTORING_REPORT.md    项目重构报告
│
├── deployment/                        ← 部署文档
│   ├── DEPLOYMENT_GUIDE.md              部署指南
│   ├── ENV_CONFIG.md                    环境配置
│   └── MONITORING_STATUS.md             监控状态
│
├── guides/                            ← 使用指南
│   ├── ENV_CONFIGURATION_GUIDE.md       环境配置指南
│   ├── LOGGING_STANDARD.md              日志规范
│   ├── DOCUMENTATION_REORGANIZATION.md  文档重组
│   └── verification.md                  验证指南
│
├── mcp/                               ← MCP相关文档
│   ├── README.md                        MCP文档导航
│   ├── mcp-prewarm.md                   MCP预热
│   └── mcp-troubleshooting.md           MCP故障排除
│
└── projects/                          ← 项目特定文档
    ├── README.md                        项目文档导航
    └── nof1-prompts.md                  nof1.ai 提示词
```

---

## 🎯 子项目文档

### 1. Rust Trading Bot

📁 `apps/rust-trading-bot/docs/`

#### 用户指南
- [快速开始](../apps/rust-trading-bot/docs/user-guide/QUICKSTART.md)
- [多交易所使用](../apps/rust-trading-bot/docs/user-guide/README_MULTI_EXCHANGE.md)
- [DeepSeek快速启动](../apps/rust-trading-bot/docs/user-guide/DEEPSEEK_GATE_QUICKSTART.md)
- [多币种交易指南](../apps/rust-trading-bot/docs/user-guide/MULTI_COIN_TRADING_GUIDE.md)
- [项目说明](../apps/rust-trading-bot/README.md)

#### 技术文档
- [区块链钱包](../apps/rust-trading-bot/docs/technical/BLOCKCHAIN_WALLETS.md)
- [Hyperliquid 集成](../apps/rust-trading-bot/docs/technical/HYPERLIQUID_README.md)
- [系统架构](../apps/rust-trading-bot/docs/technical/SYSTEM_ARCHITECTURE.md)
- [DeepSeek Rust V3 升级](../apps/rust-trading-bot/docs/technical/DEEPSEEK_RUST_V3_UPGRADE.md)

#### 优化报告
- [最终优化报告](../apps/rust-trading-bot/docs/optimization/FINAL_OPTIMIZATION_REPORT.md)
- [优化总结](../apps/rust-trading-bot/docs/optimization/OPTIMIZATION_SUMMARY.md)
- [项目清理总结](../apps/rust-trading-bot/docs/optimization/PROJECT_CLEANUP_SUMMARY.md)

#### DeepSeek AI 交易机器人
- [DeepSeek 文档中心](../apps/rust-trading-bot/docs/deepseek/README.md) ⭐
- [迁移成功报告](../apps/rust-trading-bot/docs/deepseek/DEEPSEEK_RUST_MIGRATION_SUCCESS.md)
- [使用手册](../apps/rust-trading-bot/docs/deepseek/DEEPSEEK_TRADER_README.md)

---

### 2. Social Monitor

📁 `apps/social-monitor/docs/`

- [项目说明](../apps/social-monitor/README.md)

---

### 3. DeepSeek (Python 原版)

📁 `apps/ds/docs/`

- [项目说明](../apps/ds/README.md)

---

## 📊 文档分类索引

### 按主题分类

#### 🏗️ 架构设计
- [系统架构](./architecture/ARCHITECTURE.md)
- [技术栈](../apps/rust-trading-bot/docs/technical/SYSTEM_ARCHITECTURE.md)

#### 🔒 安全相关
- [安全分析](./security/SECURITY_ANALYSIS.md)
- [安全总结](./security/SECURITY_SUMMARY.md)
- [环境变量安全](./deployment/ENV_CONFIG.md)

#### 🚀 性能优化
- [项目优化报告](./optimization/WEB3_PROJECT_OPTIMIZATION.md)
- [优化完成总结](./optimization/OPTIMIZATION_COMPLETE.md)
- [阶段2性能报告](./optimization/PHASE_2_PERFORMANCE_REPORT.md)
- [阶段3智能报告](./optimization/PHASE_3_INTELLIGENCE_REPORT.md)

#### 🛠️ 部署运维
- [部署指南](./deployment/DEPLOYMENT_GUIDE.md)
- [环境配置](./deployment/ENV_CONFIG.md)
- [监控状态](./deployment/MONITORING_STATUS.md)
- [MCP预热配置](./mcp/mcp-prewarm.md)
- [MCP故障排查](./mcp/mcp-troubleshooting.md)

#### 📖 使用指南
- [快速开始](../apps/rust-trading-bot/docs/user-guide/QUICKSTART.md)
- [环境配置指南](./guides/ENV_CONFIGURATION_GUIDE.md)
- [日志规范](./guides/LOGGING_STANDARD.md)
- [验证指南](./guides/verification.md)
- [文档重组说明](./guides/DOCUMENTATION_REORGANIZATION.md)

#### 🤖 AI 交易
- [DeepSeek 文档中心](../apps/rust-trading-bot/docs/deepseek/README.md)
- [迁移分析](../apps/rust-trading-bot/docs/deepseek/RUST_MIGRATION_ANALYSIS.md)
- [使用手册](../apps/rust-trading-bot/docs/deepseek/DEEPSEEK_TRADER_README.md)

---

## 🎯 推荐阅读路径

### 路径 1: 新用户快速上手 (15 分钟)

1. [项目总览](../README.md)
2. [快速开始](../apps/rust-trading-bot/docs/user-guide/QUICKSTART.md)
3. [环境配置](./guides/ENV_CONFIGURATION_GUIDE.md)

### 路径 2: 技术深入了解 (45 分钟)

1. [系统架构](./architecture/ARCHITECTURE.md)
2. [技术架构](../apps/rust-trading-bot/docs/technical/SYSTEM_ARCHITECTURE.md)
3. [安全分析](./security/SECURITY_ANALYSIS.md)
4. [优化报告](./optimization/WEB3_PROJECT_OPTIMIZATION.md)

### 路径 3: 部署和运维 (30 分钟)

1. [部署指南](./deployment/DEPLOYMENT_GUIDE.md)
2. [环境配置](./deployment/ENV_CONFIG.md)
3. [监控状态](./deployment/MONITORING_STATUS.md)
4. [日志规范](./guides/LOGGING_STANDARD.md)

### 路径 4: AI 交易机器人 (20 分钟)

1. [DeepSeek 文档中心](../apps/rust-trading-bot/docs/deepseek/README.md)
2. [使用手册](../apps/rust-trading-bot/docs/deepseek/DEEPSEEK_TRADER_README.md)
3. [迁移成功报告](../apps/rust-trading-bot/docs/deepseek/DEEPSEEK_RUST_MIGRATION_SUCCESS.md)

---

## 📈 项目统计

### 代码规模
- **Rust 代码**: ~15,000 行
- **TypeScript 代码**: ~3,000 行
- **Python 代码**: ~2,000 行
- **总计**: ~20,000 行

### 文档规模
- **文档数量**: 50+ 份
- **总字数**: ~100,000 字
- **文档大小**: ~1.5 MB

### 功能模块
- **交易机器人**: 5+ 交易所支持
- **区块链钱包**: 3 条链支持
- **社交监控**: Twitter 信号监控
- **AI 交易**: DeepSeek 集成

---

## 🔧 工具和脚本

### 常用脚本

```bash
# 运行交易机器人
cd apps/rust-trading-bot
cargo run --release --bin show_assets

# 运行 DeepSeek 交易机器人
cargo run --release --bin deepseek_trader

# 社交监控
cd apps/social-monitor
npm start
```

### 文档维护

```bash
# 查看所有文档
find docs -name "*.md" | sort

# 搜索文档内容
grep -r "关键词" docs/

# 文档字数统计
wc -w docs/**/*.md
```

---

## 💡 常见问题

### Q1: 如何开始使用这个项目？
查看 [快速开始指南](../apps/rust-trading-bot/docs/user-guide/QUICKSTART.md)

### Q2: 系统架构是什么样的？
查看 [系统架构文档](./architecture/ARCHITECTURE.md)

### Q3: 如何配置环境变量？
查看 [环境配置指南](./guides/ENV_CONFIGURATION_GUIDE.md)

### Q4: 安全性如何保证？
查看 [安全分析报告](./security/SECURITY_ANALYSIS.md)

### Q5: 性能如何优化的？
查看 [项目优化报告](./optimization/WEB3_PROJECT_OPTIMIZATION.md)

### Q6: DeepSeek AI 交易机器人怎么用？
查看 [DeepSeek 使用手册](../apps/rust-trading-bot/docs/deepseek/DEEPSEEK_TRADER_README.md)

---

## 🔗 外部资源

### 交易所文档
- [Binance API](https://binance-docs.github.io/apidocs/)
- [OKX API](https://www.okx.com/docs-v5/)
- [Bybit API](https://bybit-exchange.github.io/docs/)

### 技术文档
- [Rust 文档](https://doc.rust-lang.org/)
- [Tokio 异步](https://tokio.rs/)
- [Node.js](https://nodejs.org/)

### AI 服务
- [DeepSeek API](https://api.deepseek.com/)

---

## 📞 技术支持

### 文档反馈
如发现文档问题或有改进建议，请提交 Issue。

### 更新日志
- **2025-10-26**: 完成文档结构重组
- **2025-10-26**: 新增 DeepSeek AI 交易机器人文档
- **2025-10-20**: 完成项目优化报告

---

## 🎯 文档维护规范

### 文件命名
- 使用大写 + 下划线: `SYSTEM_ARCHITECTURE.md`
- 使用描述性名称
- 避免中文文件名

### 文档结构
- 包含清晰的标题层级
- 提供目录导航
- 使用 Markdown 标准格式

### 更新频率
- 重大功能: 及时更新
- 优化改进: 每周汇总
- 例行维护: 每月检查

---

**📚 欢迎探索 Web3 Trading Project 文档库！**

**快速开始**: [QUICKSTART.md](../apps/rust-trading-bot/docs/user-guide/QUICKSTART.md)

---

_文档中心最后更新: 2025-10-26_
