# 🔧 环境配置完整指南

> **统一配置管理** - Web3 Monorepo环境变量配置最佳实践  
> **重要提示**: 早期 Crypto Bot 配置现已归档，如需历史配置请参考仓库标签 `legacy/crypto-bot`.

## 📋 配置架构概览

```
Web3/
├── .env                          # 🔴 主配置文件 (生产环境)
├── .env.example                  # 📋 配置模板
└── apps/
    └── kronos-defi/packages/
        ├── trading-engine/
        │   ├── .env.example      # 📋 交易引擎模板
        │   ├── .env.small-capital # 📊 小资金策略配置
        │   └── (运行时创建 .env)
        └── ai-predictor/
            ├── .env.example      # 📋 AI预测器模板
            ├── .env.testnet      # 🧪 测试网配置
            ├── .env.production   # 🚀 生产配置
            └── (运行时创建 .env)
```

## 🎯 配置优先级

### 主配置 (根目录 `.env`)
所有服务的**默认配置**，包含：
- 数据库连接
- API密钥
- 端口配置
- 日志级别
- 通用参数

### 服务专用配置
特定服务的**覆盖配置**：
- `apps/kronos-defi/packages/trading-engine/.env` - 交易引擎专用
- `apps/kronos-defi/packages/ai-predictor/.env` - AI引擎专用

**配置合并逻辑**:
```
最终配置 = 根配置 + 服务配置 (服务配置优先)
```

## 📁 配置文件说明

### 1. 根配置文件

#### `.env` (需要创建)
```bash
# 从模板复制
cp .env.example .env
nano .env
```

**用途**: 生产环境主配置

#### `.env.example` (模板)
**用途**: 配置示例和文档，包含所有必需字段

### 2. 交易引擎配置

#### `.env.small-capital` (策略配置)
**用途**: 小资金高收益激进策略
**适用场景**:
- 初始资金 10 USDT
- 10倍杠杆
- 高频交易模式
- 激进Kelly系数

**使用方法**:
```bash
cd apps/kronos-defi/packages/trading-engine
cp .env.small-capital .env
./start.sh paper
```

### 3. AI预测器配置

#### `.env.testnet` (测试配置)
**用途**: Binance测试网验证
**适用场景**:
- 策略验证
- 无资金风险
- 激进参数测试

**使用方法**:
```bash
cd apps/kronos-defi/packages/ai-predictor
cp .env.testnet .env
python kronos_predictor.py
```

#### `.env.production` (生产配置)
**用途**: 生产环境AI配置
**适用场景**:
- 真实交易
- 保守参数
- 完整监控

## 🚀 快速开始

### 方式一: 统一配置 (推荐)

```bash
# 1. 创建根配置
cp .env.example .env
nano .env  # 编辑填入你的密钥

# 2. 启动所有服务
./start.sh

# ✅ 所有服务自动读取根配置
```

### 方式二: 自定义策略

```bash
# 1. 创建根配置
cp .env.example .env

# 2. 选择交易策略
cd apps/kronos-defi/packages/trading-engine
cp .env.small-capital .env  # 或使用 .env.example

# 3. 配置AI引擎
cd ../ai-predictor
cp .env.testnet .env        # 或使用 .env.production

# 4. 启动服务
cd ../../../../
./start.sh
```

## 🔐 敏感信息管理

### 必需的API密钥

#### Binance API (交易必需)
```env
BINANCE_API_KEY=your_api_key_here
BINANCE_SECRET_KEY=your_secret_key_here
TESTNET=true  # 建议先用测试网
```

**获取方式**:
- 测试网: https://testnet.binancefuture.com
- 主网: https://www.binance.com/en/my/settings/api-management

#### Telegram Bot (监控必需)
```env
TELEGRAM_BOT_TOKEN=your_bot_token
TELEGRAM_API_ID=your_api_id
TELEGRAM_API_HASH=your_api_hash
```

**获取方式**:
- Bot Token: @BotFather
- API ID/Hash: https://my.telegram.org/apps

#### Discord Bot (可选)
```env
DISCORD_BOT_TOKEN=your_discord_token
```

**获取方式**: https://discord.com/developers/applications

### 安全最佳实践

```bash
# ❌ 永远不要提交 .env 文件
git add .env           # 不要这样做！

# ✅ .gitignore 已配置
.env
**/.env
!.env.example

# ✅ 设置文件权限
chmod 600 .env
chmod 600 apps/*/packages/*/.env

# ✅ 定期轮换密钥
# 每30天更换一次API密钥
```

## 📊 配置场景示例

### 场景1: 纸上交易验证 (零风险)
```bash
# 根配置
TRADING_MODE=paper
INITIAL_BALANCE=1000
TESTNET=false  # 纸上交易不需要API

# 启动
./start.sh
```

### 场景2: 测试网验证 (小风险)
```bash
# 根配置
TESTNET=true
BINANCE_API_KEY=testnet_key
BINANCE_SECRET_KEY=testnet_secret

# 交易引擎
cd apps/kronos-defi/packages/trading-engine
cp .env.example .env
# 编辑 TESTNET=true

# 启动
./start.sh
```

### 场景3: 生产环境 (真实交易)
```bash
# ⚠️ 警告: 真实资金风险

# 根配置
TRADING_MODE=live
TESTNET=false
BINANCE_API_KEY=real_api_key
BINANCE_SECRET_KEY=real_secret_key

# 使用保守配置
cd apps/kronos-defi/packages/trading-engine
cp .env.example .env
# 编辑设置保守参数:
# MAX_LEVERAGE=3
# CONFIDENCE_THRESHOLD=75
# MAX_DAILY_LOSS=50

# 启动
./start.sh
```

### 场景4: 小资金激进策略
```bash
# 根配置
TRADING_MODE=paper  # 先纸上交易验证！
INITIAL_BALANCE=10

# 使用激进配置
cd apps/kronos-defi/packages/trading-engine
cp .env.small-capital .env

# 验证7天后再考虑真实交易
./start.sh paper
```

## 🛠️ 配置验证

### 检查配置完整性
```bash
# 验证根配置
cat .env | grep -E "BINANCE_API_KEY|TELEGRAM_BOT_TOKEN"

# 验证交易引擎配置
cd apps/kronos-defi/packages/trading-engine
node -e "require('dotenv').config(); console.log(process.env.SYMBOL)"

# 验证AI预测器配置
cd apps/kronos-defi/packages/ai-predictor
python -c "import os; from dotenv import load_dotenv; load_dotenv(); print(os.getenv('KRONOS_DEVICE'))"
```

### 测试API连接
```bash
# 测试Binance连接
cd apps/kronos-defi/packages/trading-engine
node test-binance-api.js

# 测试Telegram连接
cd apps/crypto-bot/collector
python test_telegram.py
```

## 🔄 配置更新流程

### 添加新配置项

1. **更新 `.env.example`**
```bash
# 添加新配置项和说明
echo "NEW_FEATURE_ENABLED=false  # 新功能开关" >> .env.example
```

2. **更新文档**
```bash
# 在本文档中添加说明
# 通知团队成员
```

3. **添加默认值**
```typescript
// 代码中提供默认值
const newFeature = process.env.NEW_FEATURE_ENABLED === 'true' || false;
```

### 弃用配置项

1. **标记为弃用**
```bash
# .env.example
# DEPRECATED: OLD_CONFIG=value  # 使用 NEW_CONFIG 替代
NEW_CONFIG=value
```

2. **保持向后兼容**
```typescript
// 代码中兼容旧配置
const config = process.env.NEW_CONFIG || process.env.OLD_CONFIG;
```

3. **3个月后移除**

## 📈 环境变量清单

### 完整变量列表

#### 核心配置
- `NODE_ENV` - 运行环境 (development/production/paper)
- `LOG_LEVEL` - 日志级别 (debug/info/warn/error)
- `TRADING_MODE` - 交易模式 (paper/futures/live)

#### API密钥
- `BINANCE_API_KEY` - Binance API密钥
- `BINANCE_SECRET_KEY` - Binance Secret密钥
- `TELEGRAM_BOT_TOKEN` - Telegram Bot Token
- `TELEGRAM_API_ID` - Telegram API ID
- `TELEGRAM_API_HASH` - Telegram API Hash
- `DISCORD_BOT_TOKEN` - Discord Bot Token

#### 交易参数
- `SYMBOL` - 交易对 (ETHUSDT)
- `TRADE_AMOUNT` - 单笔交易金额
- `MAX_LEVERAGE` - 最大杠杆倍数
- `CONFIDENCE_THRESHOLD` - AI置信度阈值
- `STOP_LOSS_PCT` - 止损比例
- `TAKE_PROFIT_PCT` - 止盈比例

#### 风险控制
- `MAX_DAILY_LOSS` - 日最大亏损限制
- `MAX_POSITION_SIZE` - 最大仓位大小
- `KELLY_CONSERVATIVE_FACTOR` - Kelly保守系数
- `KELLY_MAX_POSITION` - Kelly最大仓位

#### AI配置
- `KRONOS_DEVICE` - AI设备 (auto/cpu/cuda)
- `KRONOS_PRED_LEN` - 预测长度
- `PREDICTION_INTERVAL` - 预测间隔

#### 服务端口
- `CRYPTO_BOT_PORT=8080` - Crypto Bot端口
- `SOCIAL_MONITOR_PORT=3002` - Social Monitor端口
- `KRONOS_TRADING_PORT=4567` - Trading引擎端口
- `KRONOS_AI_PORT=4568` - AI预测器端口

### 详细配置说明

请参考各服务的 `.env.example` 文件获取完整配置说明。

## 🆘 故障排除

### 配置未生效
```bash
# 检查文件是否存在
ls -la .env

# 检查文件权限
chmod 600 .env

# 检查语法错误
cat .env | grep -v "^#" | grep "="

# 重启服务
./start.sh
```

### API连接失败
```bash
# 检查密钥配置
cat .env | grep BINANCE_API

# 测试连接
cd apps/kronos-defi/packages/trading-engine
node test-binance-api.js

# 检查IP白名单
# 确保服务器IP在Binance API白名单中
```

### 服务无法启动
```bash
# 检查所有必需配置
./start.sh
# 选项 4: 测试API连接

# 查看服务日志
docker-compose logs -f

# 检查端口冲突
netstat -tlnp | grep -E "8080|3002|4567|4568"
```

## 📞 支持

### 配置相关问题
1. 查看 `.env.example` 文件中的注释
2. 参考本文档中的场景示例
3. 运行配置验证脚本
4. 查看服务日志获取错误信息

### 更新日志
- 2025-09-29: 初始版本，整合所有配置文档
- Phase 1: 统一配置管理架构
- Phase 2: 性能优化相关配置
- Phase 3: 智能化功能配置

---

## 💡 最佳实践总结

✅ **推荐做法**:
- 使用根目录 `.env` 作为主配置
- 先用纸上交易验证策略
- 定期备份配置文件
- 使用测试网验证API连接
- 设置严格的文件权限

❌ **避免做法**:
- 不要提交 `.env` 文件到Git
- 不要在代码中硬编码密钥
- 不要跳过纸上交易直接真实交易
- 不要使用过高杠杆
- 不要忽略风险警告

---

**这是"好品味"的配置管理: 简洁、安全、可维护。**
