# 📚 Rust Trading Bot - 文档索引

> 最后更新：2025-10-30

---

## 🚀 快速开始

如果你是新用户，请按以下顺序阅读：

1. **[README.md](README.md)** - 项目总览
2. 根据你的需求选择：
   - **主力资金追踪** → [QUICKSTART_SMART_MONEY.md](QUICKSTART_SMART_MONEY.md)
   - **传统跟单系统** → [QUICKSTART.md](QUICKSTART.md)

---

## 📁 文档分类

### 🎯 主力资金追踪系统（最新）

| 文档 | 说明 | 适用对象 |
|-----|------|---------|
| [SMART_MONEY_STRATEGY.md](SMART_MONEY_STRATEGY.md) | 策略设计文档 | 开发者/策略研究 |
| [QUICKSTART_SMART_MONEY.md](QUICKSTART_SMART_MONEY.md) | 快速启动指南 | **新用户必读** |
| [IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md) | 实现总结 | 开发者 |

**核心特性**：
- ✅ 基于主力资金流向信号
- ✅ 1小时K线技术分析
- ✅ 动态支撑阻力位识别
- ✅ 智能信号优先级评估

---

### 🔧 技术实现

| 文档 | 说明 | 适用对象 |
|-----|------|---------|
| [TECHNICAL_INDICATORS_ONLY.md](TECHNICAL_INDICATORS_ONLY.md) | 纯技术指标版本升级说明 | 开发者 |
| [docs/technical/SYSTEM_ARCHITECTURE.md](docs/technical/SYSTEM_ARCHITECTURE.md) | 系统架构 | 架构师 |

---

### 📖 传统跟单系统（已有功能）

| 文档 | 说明 | 适用对象 |
|-----|------|---------|
| [QUICKSTART.md](QUICKSTART.md) | 传统跟单快速启动 | 跟单用户 |
| [docs/user-guide/README_MULTI_EXCHANGE.md](docs/user-guide/README_MULTI_EXCHANGE.md) | 多交易所支持 | 高级用户 |

---

### 🤖 DeepSeek AI 交易

| 文档 | 说明 | 适用对象 |
|-----|------|---------|
| [docs/deepseek/README.md](docs/deepseek/README.md) | DeepSeek AI总览 | 新用户 |
| [docs/deepseek/DEEPSEEK_TRADER_README.md](docs/deepseek/DEEPSEEK_TRADER_README.md) | AI交易详细说明 | 用户 |

---

## 🗂️ 文档目录结构

```
rust-trading-bot/
├── README.md                              # 项目主文档
├── DOC_INDEX.md                           # 📌 本文档索引
│
├── 主力资金追踪系统/
│   ├── SMART_MONEY_STRATEGY.md           # 策略设计
│   ├── QUICKSTART_SMART_MONEY.md         # 快速启动
│   └── IMPLEMENTATION_SUMMARY.md         # 实现总结
│
├── 技术说明/
│   ├── TECHNICAL_INDICATORS_ONLY.md      # 技术指标版本
│   └── SYSTEM_ARCHITECTURE.md            # 系统架构
│
├── 传统功能/
│   └── QUICKSTART.md                     # 跟单系统快速启动
│
└── docs/                                 # 详细文档目录
    ├── README.md                         # docs 总览
    ├── deepseek/                         # DeepSeek AI (2个)
    │   ├── README.md
    │   └── DEEPSEEK_TRADER_README.md
    │
    ├── technical/                        # 技术文档 (3个)
    │   ├── SYSTEM_ARCHITECTURE.md
    │   ├── BLOCKCHAIN_WALLETS.md
    │   └── HYPERLIQUID_README.md
    │
    └── user-guide/                       # 用户指南 (1个)
        └── README_MULTI_EXCHANGE.md
```

---

## 🎯 按使用场景查找文档

### 场景1：我想使用主力资金追踪交易

1. 阅读策略说明：[SMART_MONEY_STRATEGY.md](SMART_MONEY_STRATEGY.md)
2. 快速启动系统：[QUICKSTART_SMART_MONEY.md](QUICKSTART_SMART_MONEY.md)
3. 了解实现细节：[IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md)

### 场景2：我想使用传统跟单功能

1. 快速启动：[QUICKSTART.md](QUICKSTART.md)
2. 多交易所配置：[docs/user-guide/README_MULTI_EXCHANGE.md](docs/user-guide/README_MULTI_EXCHANGE.md)

### 场景3：我想使用 DeepSeek AI 交易

1. 总览：[docs/deepseek/README.md](docs/deepseek/README.md)
2. 详细说明：[docs/deepseek/DEEPSEEK_TRADER_README.md](docs/deepseek/DEEPSEEK_TRADER_README.md)
3. 技术指标版本：[TECHNICAL_INDICATORS_ONLY.md](TECHNICAL_INDICATORS_ONLY.md)

### 场景4：我是开发者，想了解系统架构

1. 系统架构：[docs/technical/SYSTEM_ARCHITECTURE.md](docs/technical/SYSTEM_ARCHITECTURE.md)
2. 技术实现：[IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md)


---

## 📌 文档更新记录

### 2025-10-30
- ✅ 新增主力资金追踪系统文档
- ✅ 创建文档索引（本文件）
- ✅ 移除市场情绪分析模块
- ✅ 优化技术指标版本
- ✅ 清理过时文档（删除 INTEGRATION_NOTES、archive、optimization、analysis）

---

## 🔍 快速搜索

### 关键词索引

| 关键词 | 相关文档 |
|-------|---------|
| **主力资金** | SMART_MONEY_STRATEGY.md, QUICKSTART_SMART_MONEY.md |
| **支撑阻力位** | SMART_MONEY_STRATEGY.md, IMPLEMENTATION_SUMMARY.md |
| **技术指标** | TECHNICAL_INDICATORS_ONLY.md |
| **跟单** | QUICKSTART.md, README_MULTI_EXCHANGE.md |
| **DeepSeek** | docs/deepseek/*.md |
| **Telegram** | QUICKSTART.md |
| **交易所** | README_MULTI_EXCHANGE.md, HYPERLIQUID_README.md |

---

## 💡 贡献指南

### 添加新文档

1. 根据文档类型放入对应目录
2. 在本索引中添加链接
3. 更新文档更新记录

### 文档规范

- 使用 Markdown 格式
- 包含清晰的标题和目录
- 代码示例使用语法高亮
- 添加表情符号增强可读性

---

## 📞 获取帮助

- **问题反馈**：GitHub Issues
- **功能建议**：GitHub Discussions
- **紧急咨询**：查看各文档中的联系方式

---

**提示**：建议将本文档加入书签，方便快速查找所需文档。
