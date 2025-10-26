# 环境变量配置指南

## 📋 概述

Web3 Monorepo 现在使用**统一的环境变量配置**，所有服务共享根目录的 `.env` 文件。

## 🚀 快速开始

### 1. 复制环境变量模板
```bash
cp .env.example .env
```

### 2. 编辑配置文件
```bash
nano .env
```

### 3. 启动服务
```bash
./start.sh
```

## 📂 文件结构

```
Web3/
├── .env                    # 📌 统一环境变量文件 (你需要创建)
├── .env.example           # 📋 环境变量模板
├── start.sh               # 🚀 统一启动脚本
├── apps/
│   ├── rust-trading-bot/   # 🤖 Rust交易机器人
│   ├── social-monitor/     # 📱 社交媒体监控
│   └── kronos-defi/       # 💹 DeFi交易系统
└── README.md              # 📖 项目说明
```

## 🔧 环境变量分类

### 🗄️ 数据库配置
- `DATABASE_URL` - PostgreSQL连接字符串
- `REDIS_URL` - Redis连接字符串

### 🤖 Telegram配置
- `TELEGRAM_BOT_TOKEN` - Telegram机器人Token
- `TELEGRAM_API_ID` - Telegram API ID
- `TELEGRAM_API_HASH` - Telegram API Hash

### 💱 Binance配置
- `BINANCE_API_KEY` - Binance API密钥
- `BINANCE_SECRET_KEY` - Binance Secret密钥
- `BINANCE_TESTNET` - 是否使用测试网 (true/false)

### 📊 交易参数
- `SYMBOL` - 交易对 (默认: ETHUSDT)
- `TRADE_AMOUNT` - 单笔交易金额
- `MAX_LEVERAGE` - 最大杠杆倍数
- `CONFIDENCE_THRESHOLD` - AI置信度阈值

### ⚙️ 服务端口
- `NITTER_PORT` - Nitter服务端口 (默认: 3001)
- `SOCIAL_MONITOR_PORT` - 社交监控端口 (默认: 3002)
- `CRYPTO_BOT_PORT` - 加密货币机器人端口 (默认: 8080)

## 🛠️ 服务启动

### 方式一: 使用统一启动脚本 (推荐)
```bash
./start.sh
```

### 方式二: 单独启动服务

#### Rust交易机器人
```bash
cd apps/rust-trading-bot
./start.sh
```

#### 社交媒体监控
```bash
cd apps/social-monitor
docker-compose up -d
```

#### Kronos DeFi交易
```bash
cd apps/kronos-defi/packages/trading-engine
./start.sh
```

## 🔍 环境变量验证

测试Binance API连接:
```bash
node apps/rust-trading-bot/test-binance-api.js
```

## 📱 访问地址

- **Nitter**: http://localhost:3001
- **社交监控面板**: http://localhost:3002
- **加密货币机器人**: http://localhost:8080

## ⚠️ 重要提醒

1. **安全**: 永远不要提交真实的 `.env` 文件到Git
2. **测试网**: 建议先使用测试网进行测试
3. **API权限**: 确保Binance API有合约交易权限
4. **IP白名单**: 确保API密钥的IP白名单配置正确

## 🆘 故障排除

### 环境变量未加载
```bash
# 检查.env文件是否存在
ls -la .env

# 检查文件内容
cat .env
```

### 服务无法启动
```bash
# 查看服务状态
./start.sh
# 选择选项 6 查看服务状态
```

### API连接失败
```bash
# 测试API连接
./start.sh
# 选择选项 4 测试API连接
```

## 📞 获取帮助

如果遇到问题，请：
1. 检查 `.env` 文件配置
2. 查看服务日志
3. 运行API连接测试
4. 查看端口是否被占用

---

## 📋 完整配置示例

参考 `.env.example` 文件获取完整的配置模板。