# 🌐 Web3 Monorepo

一个现代化的Web3项目集合，聚焦社交媒体情报监控、AI 驱动的 DeFi 交易以及 Rust 生态的量化机器人。

## 📚 核心文档

**新手必读**:
- [📖 系统架构总览](./ARCHITECTURE.md) - 完整架构说明、技术选型、性能指标
- [⚙️ 环境配置指南](./docs/ENV_CONFIGURATION_GUIDE.md) - 统一配置管理、场景示例
- [📝 日志标准](./docs/LOGGING_STANDARD.md) - 统一日志格式、搜索分析

**演进报告**:
- [Phase 1: 架构优化](./docs/OPTIMIZATION_REPORT.md) - 37脚本→13脚本，标准化
- [Phase 2: 性能提升](./docs/PHASE_2_PERFORMANCE_REPORT.md) - 6.2x API提升，12.8x查询优化
- [Phase 3: 智能化升级](./docs/PHASE_3_INTELLIGENCE_REPORT.md) - 23维AI预测，月收益翻倍

**运维指南**:
- [🚀 部署指南](./DEPLOYMENT_GUIDE.md)
- [🔧 环境变量配置](./ENV_CONFIG.md)

## 🏗️ 项目结构

```
Web3/
├── apps/
│   ├── social-monitor/      # 统一社交媒体监控平台
│   │   ├── services/
│   │   │   ├── nitter/      # Twitter监控 (Nitter)
│   │   │   ├── telegram/    # Telegram监控
│   │   │   ├── discord/     # Discord监控
│   │   │   └── aggregator/  # 数据聚合API
│   │   └── config/          # 监控配置
│   ├── kronos-defi/         # 🚀 Kronos AI驱动DeFi交易系统
│       ├── packages/        # 核心功能包
│       │   ├── ai-predictor/     # Python AI预测引擎
│       │   ├── contracts/        # 多链智能合约
│       │   ├── trading-engine/   # TypeScript交易引擎
│       │   ├── twitter-monitor/  # 6551.io Twitter监控
│       │   └── core/            # 共享工具库
│       └── apps/           # Web Dashboard
│   └── rust-trading-bot/    # Rust 实盘/跟单交易机器人
├── packages/               # 共享包（预留）
├── tools/                  # 构建和开发工具
└── docs/                   # 项目文档
```

## 🚀 快速开始

### 前置要求

- Node.js 18+
- Docker & Docker Compose
- npm 9+

### 安装依赖

```bash
# 安装所有依赖
npm install

# 或者使用yarn
yarn install
```

### 启动服务

```bash
# 启动所有核心服务（Docker）
npm run start

# 启动单个服务
npm run start:social-monitor
npm run kronos:dev

# 开发模式
npm run dev                   # 启动所有应用的开发模式
npm run social-monitor:dev    # 启动社交媒体监控开发模式
npm run kronos:dev            # 启动Kronos DeFi交易系统开发模式
npm run kronos:twitter        # 启动Kronos Twitter监控 (6551.io)
```

## 📦 应用介绍

### 🦀 Rust Trading Bot (`apps/rust-trading-bot`)

**Binance 合约复制交易与 Telegram 控制中心**

- ✅ **双账户联动**: 同时管理带单者与跟单账户
- ✅ **风险控制**: 最大仓位、止损比例、杠杆上限
- ✅ **Telegram Bot**: 支持启动/停止、账户状态、持仓、统计、比例调整
- ✅ **异步执行**: Tokio + Reqwest 定时同步与下单
- ✅ **测试工具**: `test-binance-api.js` 快速验证 API 密钥

**技术栈**:
- 语言: Rust (tokio / reqwest / teloxide)
- 交易所: Binance Futures API
- 日志: env_logger
- 部署: systemd / Docker 自行选择

### 🌐 Social Monitor (`apps/social-monitor`)

**统一的社交媒体监控平台**

- ✅ **多平台集成**: Twitter (Nitter) + Telegram + Discord
- ✅ **实时数据聚合**: 统一API和WebSocket推送
- ✅ **智能过滤**: 关键词检测和垃圾信息过滤
- ✅ **优先级排序**: 基于价值和紧急度的智能排序
- ✅ **可扩展架构**: 微服务设计，易于添加新平台
- ✅ **企业级监控**: 健康检查和性能监控

**服务组成**:
- **Nitter服务**: 隐私友好的Twitter监控
- **Telegram监控**: 群组和频道实时监控
- **Discord监控**: 服务器消息监控
- **数据聚合器**: 统一API和实时推送
- **监控面板**: 可视化数据展示

**技术栈**:
- 后端: Node.js + TypeScript + Express
- 监控: Python异步爬虫 + Telegram/Discord Bot
- 缓存: Redis + 发布订阅
- 前端: React监控面板
- 容器化: Docker Compose微服务

### 🚀 Kronos DeFi (`apps/kronos-defi`)

**AI驱动的多链去中心化预测交易平台**

- ✅ **Kronos AI模型**: 65-78%方向预测准确率
- ✅ **多链智能合约**: 7个区块链(Ethereum, Solana, Base等)
- ✅ **6551.io数据流**: 实时Twitter情绪分析和交易信号
- ✅ **机构级策略**: Kelly公式仓位管理和专业做市
- ✅ **现代化架构**: Python AI + TypeScript引擎 + React Dashboard

**核心组件**:
- **AI预测引擎**: Python + 100+技术指标
- **交易引擎**: TypeScript自动化执行
- **智能合约**: Solidity + Rust多链部署
- **Twitter监控**: 6551.io WebSocket实时数据流
- **Web控制台**: React + tRPC实时监控

**技术栈**:
- AI: Python + NumPy + 机器学习
- 后端: Node.js + TypeScript + tRPC
- 前端: React + TypeScript + Tailwind
- 区块链: Solidity + Rust + Move
- 数据: SQLite + Redis + WebSocket

## 🛠️ 开发命令

```bash
# 构建所有应用
npm run build

# 运行测试
npm run test

# 代码检查
npm run lint

# 类型检查
npm run type-check

# 代码格式化
npm run format

# 清理构建文件
npm run clean
```

## 📊 Monorepo管理

此项目使用 [Turbo](https://turbo.build/) 进行monorepo管理：

- **并行构建**: 多个应用同时构建
- **增量构建**: 只构建更改的部分
- **缓存优化**: 智能缓存机制
- **依赖图管理**: 自动处理依赖关系

## 🔧 配置文件

- `package.json` - 根级依赖和脚本
- `turbo.json` - Turbo构建配置
- `.gitignore` - Git忽略规则
- `tsconfig.json` - TypeScript配置

## 📈 架构特点

### 📡 **实时情报采集**
- 多平台社交数据整合（Telegram / Discord / Twitter）
- Redis 驱动的事件推送与优先级队列
- WebSocket 广播最新机会与系统状态

### 🤖 **AI 交易执行**
- Kronos Trading Engine 集成外部/内置预测信号
- tRPC API 统一对接策略、面板与自动化工具
- 冷静期风控、Kelly 仓位、纸上/实盘双模式

### 🛡️ **风险与运维**
- Rust Copy-Trader 提供秒级持仓同步与 Telegram 控制
- Docker Compose 管理核心基础设施（Redis / Kronos / Monitor）
- 统一日志规范与性能指标采集

## 🌟 使用场景

1. **社交情绪监控**: 聚合 Twitter/Nitter、Telegram、Discord 事件并自动排序
2. **AI 驱动交易**: 使用 Kronos DeFi 引擎进行纸上或实盘交易实验
3. **复制交易**: Rust Bot 同步带单者持仓，实时推送至 Telegram
4. **策略研究**: 借助 `packages/advanced-features` 拓展特征工程、回测与数据可视化
5. **可视化面板**: 通过 Web Dashboard 与 WebSocket 监控交易与风控指标

## 🤝 贡献指南

1. Fork 此仓库
2. 创建功能分支: `git checkout -b feature/amazing-feature`
3. 提交更改: `git commit -m 'Add amazing feature'`
4. 推送到分支: `git push origin feature/amazing-feature`
5. 提交 Pull Request

## 📄 许可证

此项目基于 [MIT License](LICENSE) 开源。

## 🔗 相关链接

- [Social Monitor 文档](./apps/social-monitor/README.md)
- [Kronos DeFi 文档](./apps/kronos-defi/README.md)
- [Rust Trading Bot 指南](./apps/rust-trading-bot/README.md)
- [环境配置指南](./docs/ENV_CONFIGURATION_GUIDE.md)
- [部署指南](./DEPLOYMENT_GUIDE.md)

---

**⚡ 由 Web3 团队构建，为加密货币社区提供自动化工具**
