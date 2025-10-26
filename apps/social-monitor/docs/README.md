# 📱 Social Monitor - 文档中心

**Twitter 信号监控系统**

---

## 📖 项目介绍

Social Monitor 是一个 Twitter 信号监控和分析系统，用于实时监控加密货币交易信号。

---

## 🎯 功能特性

- ✅ Twitter 频道监控
- ✅ 信号解析和提取
- ✅ 实时通知
- ✅ 信号分析
- ✅ 自动交易集成

---

## 🚀 快速开始

### 安装依赖

```bash
cd apps/social-monitor
npm install
```

### 配置环境

创建 `.env.example` 并重命名为 `.env`:

```bash
# Telegram 配置
TELEGRAM_API_ID=your_api_id
TELEGRAM_API_HASH=your_api_hash
TELEGRAM_SESSION=your_session

# 监控配置
CHANNELS=channel1,channel2
```

### 运行

```bash
npm start
```

---

## 📂 项目结构

```
apps/social-monitor/
├── services/
│   ├── nitter/              # Nitter 服务
│   └── telegram/            # Telegram 监控
├── src/
│   ├── monitor.js           # 监控主程序
│   └── parser.js            # 信号解析
├── README.md
└── package.json
```

---

## 📚 相关文档

- [项目主页](../README.md)
- [项目文档中心](../../../docs/README.md)
- [Rust Trading Bot](../../rust-trading-bot/docs/README.md)

---

**📱 Social Monitor - 实时信号监控**

_最后更新: 2025-10-26_
