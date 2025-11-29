# Web3 项目配置说明

## 🔐 环境变量配置规则

**重要**: 本项目的所有环境变量配置**必须且只能**在**根目录的 `.env` 文件**中设置。

```
web3/                           ← 项目根目录
├── .env                        ← ✅ 唯一的环境变量配置文件
├── apps/
│   ├── rust-trading-bot/
│   │   └── .env               ← ❌ 不使用此文件
│   ├── nofx/
│   └── valuescan/
└── packages/
```

### 📋 配置文件位置

**正确路径**: `/home/hanins/code/web3/.env`

**错误路径**:
- ❌ `/home/hanins/code/web3/apps/rust-trading-bot/.env`
- ❌ `/home/hanins/code/web3/apps/*//.env`

### 🎯 原因说明

1. **Monorepo架构**: 本项目采用monorepo架构,多个子应用共享配置
2. **统一管理**: 所有API密钥、数据库配置等敏感信息集中管理
3. **避免冲突**: 防止多个.env文件导致配置不一致

### 🔑 重要环境变量列表

根目录 `/home/hanins/code/web3/.env` 包含以下配置:

#### Binance API (Rust交易机器人)
```bash
BINANCE_API_KEY=********
BINANCE_SECRET=*****
BINANCE_TESTNET=false
```

#### 其他交易所API
- Bitget API
- Bybit API
- OKX API
- Gate API
- Hyperliquid

#### AI服务
```bash
DEEPSEEK_API_KEY=sk-c5241fa12c4c4fa1a0d708ebc7645430
```

#### Telegram配置
```bash
TELEGRAM_API_ID=2040
TELEGRAM_API_HASH=b18441a1ff607e10a989891a5462e627
TELEGRAM_PHONE=+17578852234
```

### 📝 修改环境变量的正确步骤

1. **编辑根目录.env文件**:
   ```bash
   cd /home/hanins/code/web3
   vim .env  # 或使用其他编辑器
   ```

2. **修改对应变量**:
   ```bash
   BINANCE_API_KEY=新的密钥
   BINANCE_SECRET=新的密钥
   ```

3. **重启相关服务**:
   ```bash
   # Rust交易机器人
   cd apps/rust-trading-bot
   bash start_trader.sh
   ```

### ⚠️ 注意事项

1. **不要创建子目录的.env**: 即使子应用目录下有.env文件,也不会被读取
2. **环境变量优先级**: 只有根目录的.env会被加载
3. **敏感信息保护**: .env文件已添加到.gitignore,不会被提交到git

### 🔄 当前Binance API状态

**当前问题**: API密钥返回 `-2015` 错误 (Invalid API-key, IP, or permissions)

**解决方案**:
1. 登录 Binance → API管理
2. 找到密钥 `dpr1YD1T...`
3. 开启权限:
   - ✅ Enable Reading
   - ✅ Enable Futures
4. 保存后等待1-5分钟生效
5. 重启交易机器人

### 📂 项目结构

```
web3/
├── .env                          # ✅ 主配置文件
├── apps/
│   └── rust-trading-bot/         # Rust AI交易机器人
│       ├── src/
│       │   ├── binance_client.rs # Binance API客户端
│       │   ├── database.rs       # SQLite数据持久化
│       │   ├── web_server.rs     # Web API服务器
│       │   └── bin/
│       │       └── integrated_ai_trader.rs  # 主程序
│       ├── web/                  # 前端监控面板
│       │   ├── src/
│       │   └── vite.config.ts    # 配置API代理到localhost:8080
│       ├── data/
│       │   └── trading.db        # SQLite数据库
│       └── start_trader.sh       # 启动脚本
└── packages/
```

### 🚀 快速启动

```bash
# 1. 确保环境变量配置正确
cd /home/hanins/code/web3
cat .env | grep BINANCE

# 2. 启动Rust交易机器人
cd apps/rust-trading-bot
bash start_trader.sh

# 3. 启动前端监控面板
cd web
npm run dev

# 4. 访问监控面板
# http://localhost:5173
```

### 📊 服务端口

- **Web API**: `http://localhost:8080`
- **前端面板**: `http://localhost:5173`
- **健康检查**: `http://localhost:8080/health`

---

**最后更新**: 2025-11-09
**维护者**: Linus Torvalds (Claude Code)
