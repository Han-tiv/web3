# 🚀 快速参考指南

## 一键启动

```bash
cd /home/hantiv/code/Web3/apps/rust-trading-bot
./start.sh
```

## 核心命令速查

### Telegram Bot命令
```
/start      - 启动跟单
/stop       - 停止跟单
/status     - 账户状态
/positions  - 查看持仓
/stats      - 跟单统计
/ratio 0.5  - 设置50%跟单比例
```

## 重要配置项

### .env配置
```env
# Bot Token（必填）
TELOXIDE_TOKEN=你的Telegram_Bot_Token

# 跟单者API（必填）
BINANCE_API_KEY=你的API_Key
BINANCE_SECRET_KEY=你的Secret_Key

# 带单者API（必填，只读权限）
LEADER_API_KEY=带单者API_Key
LEADER_SECRET_KEY=带单者Secret_Key

# 跟单参数
COPY_RATIO=0.5              # 跟单比例：0.1-1.0
MAX_POSITION_SIZE=100       # 单笔最大金额USDT
LEVERAGE=3                  # 杠杆：1-125
BINANCE_TESTNET=true        # 测试网：true/false
```

## 框架选择

### Telegram Bot框架
**推荐：teloxide** ⭐⭐⭐⭐⭐
- 最流行的Rust Telegram框架
- 异步高性能
- 类型安全
- 文档完善

```toml
[dependencies]
teloxide = { version = "0.12", features = ["macros"] }
```

### Binance API框架
**推荐：binance-rs-async** ⭐⭐⭐⭐
- 官方API完整支持
- 异步实现
- WebSocket支持
- 活跃维护

```toml
[dependencies]
binance = { version = "1.3", features = ["futures-usd-m"] }
```

## 风险管理

### 建议配置
```
测试阶段：
- COPY_RATIO: 0.1 (10%)
- MAX_POSITION_SIZE: 10 USDT
- LEVERAGE: 1-2x
- BINANCE_TESTNET: true

正式交易：
- COPY_RATIO: 0.3-0.5 (30-50%)
- MAX_POSITION_SIZE: 50-100 USDT
- LEVERAGE: 2-5x
- BINANCE_TESTNET: false
```

### 止损设置
代码默认5%止损，可在 `src/copy_trader.rs` 修改：

```rust
CopyTradeConfig {
    stop_loss_percent: 0.05,  // 5%止损
    enable_stop_loss: true,
}
```

## 常见问题

### Q1: 如何获取Telegram Bot Token？
1. 在Telegram搜索 @BotFather
2. 发送 `/newbot`
3. 按提示创建，获得Token

### Q2: 如何创建Binance API？
1. 登录Binance
2. 账户 → API管理
3. 创建API Key
4. **重要**：开启"允许现货及杠杆交易"或"允许合约交易"

### Q3: 带单者API需要什么权限？
- ✅ 只读权限（查询持仓）
- ❌ 不需要交易权限
- ❌ 不需要提现权限

### Q4: 如何测试？
1. 设置 `BINANCE_TESTNET=true`
2. 在Binance测试网注册账号
3. 申请测试网API Key
4. 运行程序测试

## 性能优化

### 编译优化
```bash
# Release模式（生产环境）
cargo build --release

# 极致优化（更小体积）
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

### 监控频率调整
修改 `src/copy_trader.rs:94`:
```rust
// 默认5秒检查一次
let mut check_interval = interval(Duration::from_secs(5));

// 改为更快（1秒）
let mut check_interval = interval(Duration::from_secs(1));
```

## 项目结构

```
rust-trading-bot/
├── src/
│   ├── main.rs              # 🚪 入口
│   ├── binance_client.rs    # 📡 Binance API
│   ├── copy_trader.rs       # 🤖 跟单引擎
│   └── telegram_bot.rs      # 💬 Telegram Bot
├── Cargo.toml               # 📦 依赖
├── .env                     # ⚙️ 配置
├── start.sh                 # 🚀 启动脚本
└── README.md                # 📖 文档
```

## 依赖库说明

| 库 | 用途 | 版本 |
|---|---|---|
| teloxide | Telegram Bot框架 | 0.12 |
| binance | Binance API客户端 | 1.3 |
| tokio | 异步运行时 | 1.x |
| serde | 序列化/反序列化 | 1.0 |
| reqwest | HTTP客户端 | 0.11 |
| log/env_logger | 日志系统 | 0.4/0.11 |
| anyhow | 错误处理 | 1.0 |

## 技术支持

遇到问题？

1. 查看日志：`RUST_LOG=debug cargo run`
2. 检查配置：`cat .env`
3. 测试网络：`ping api.binance.com`
4. 查看文档：[README.md](README.md)

---

**⚠️ 风险提示**

合约交易有极高风险，可能导致本金全部损失。

- 先在测试网验证
- 小资金开始
- 严格止损
- 理性交易

祝交易顺利！🚀