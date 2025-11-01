# 🌐 Web3 Monorepo

一个现代化的Web3项目集合，聚焦社交媒体情报监控、AI 驱动的 DeFi 交易以及 Rust 生态的量化机器人。

---

## 📚 文档中心

**🎯 [完整文档导航](./docs/README.md)** ← 所有文档的统一入口

### 快速链接

**新手必读**:
- [📖 系统架构总览](./docs/architecture/ARCHITECTURE.md) - 完整架构说明、技术选型、性能指标
- [⚙️ 环境配置指南](./docs/guides/ENV_CONFIGURATION_GUIDE.md) - 统一配置管理、场景示例
- [🚀 快速开始](./apps/rust-trading-bot/docs/user-guide/QUICKSTART.md) - 5分钟上手

**演进报告**:
- [Phase 1: 架构优化](./docs/optimization/OPTIMIZATION_REPORT.md) - 37脚本→13脚本，标准化
- [Phase 2: 性能提升](./docs/optimization/PHASE_2_PERFORMANCE_REPORT.md) - 6.2x API提升，12.8x查询优化
- [Phase 3: 智能化升级](./docs/optimization/PHASE_3_INTELLIGENCE_REPORT.md) - 23维AI预测，月收益翻倍

**运维指南**:
- [🚀 部署指南](./docs/deployment/DEPLOYMENT_GUIDE.md)
- [🔒 安全分析](./docs/security/SECURITY_ANALYSIS.md)
- [📊 项目优化](./docs/optimization/WEB3_PROJECT_OPTIMIZATION.md)

**子项目文档**:
- [🦀 Rust Trading Bot 文档](./apps/rust-trading-bot/docs/README.md)
- [🤖 DeepSeek AI 交易](./apps/rust-trading-bot/docs/deepseek/README.md) ⭐ 新增
- [📱 Social Monitor 文档](./apps/social-monitor/docs/README.md)

**项目管理**:
- [📁 项目结构说明](./PROJECT_STRUCTURE.md) - 详细的目录结构说明
- [🔧 脚本使用指南](./scripts/README.md) - 所有脚本的说明
- [⚙️ 配置管理](./config/README.md) - 配置文件管理

---

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

# 启动社交监控
npm run start:social-monitor

# 开发模式
npm run dev                   # 启动所有应用的开发模式
npm run social-monitor:dev    # 启动社交媒体监控开发模式
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

### 🛡️ **风险与运维**
- Rust Copy-Trader 提供秒级持仓同步与 Telegram 控制
- Docker Compose 管理核心基础设施（Redis / Monitor）
- 统一日志规范与性能指标采集

## 🌟 使用场景

1. **社交情绪监控**: 聚合 Twitter/Nitter、Telegram、Discord 事件并自动排序
2. **复制交易**: Rust Bot 同步带单者持仓，实时推送至 Telegram
3. **策略研究**: 借助 `packages/advanced-features` 拓展特征工程、回测与数据可视化
4. **可视化面板**: 通过 Web Dashboard 与 WebSocket 监控交易与风控指标

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
- [Rust Trading Bot 指南](./apps/rust-trading-bot/README.md)
- [环境配置指南](./docs/guides/ENV_CONFIGURATION_GUIDE.md)
- [部署指南](./docs/deployment/DEPLOYMENT_GUIDE.md)

---

**⚡ 由 Web3 团队构建，为加密货币社区提供自动化工具**
