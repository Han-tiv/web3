# 🦀 Rust Trading Bot - 加密货币智能交易系统

<div align="center">

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**高性能 Rust 实现的多策略加密货币自动交易系统**

[功能特性](#功能特性) • [快速开始](#快速开始) • [文档](#文档) • [风险提示](#风险提示)

</div>

---

## ✨ 功能特性

### 🎯 主力资金追踪交易（最新）

基于主力资金流向信号的智能交易系统：

- ✅ **主力资金位识别** - 1小时K线，找最大成交量主力活跃区域
- ✅ **动态支撑阻力** - 实时计算关键价格位，强度评分0-100
- ✅ **智能信号生成** - 5种交易信号：突破/回踩/破位/平仓/持有
- ✅ **优先级评估** - Critical/High/Medium/Low 四级优先级
- ✅ **自动止损止盈** - 基于关键位动态计算，非固定百分比
- ✅ **多重确认机制** - 价格 + 成交量 + 资金流向三重验证

📖 **快速启动**: [QUICKSTART_SMART_MONEY.md](QUICKSTART_SMART_MONEY.md)

---

### 📡 传统跟单系统

- ✅ **实时跟单** - 5秒检测带单者持仓变化，自动跟随开平仓
- ✅ **智能比例** - 自定义跟单比例（10%-100%）
- ✅ **风险控制** - 最大仓位限制、自动止损
- ✅ **杠杆交易** - 支持1-125倍杠杆
- ✅ **Telegram控制** - 通过Bot实时监控和控制

📖 **详细文档**: [跟单系统快速启动](QUICKSTART.md)

---

### 🤖 DeepSeek AI 交易

- ✅ **纯技术指标版本** - RSI/MACD/布林带/均线分析
- ✅ **多周期分析** - 15分钟/1小时K线
- ✅ **AI决策引擎** - DeepSeek模型生成交易信号
- ✅ **防频繁交易** - 智能信号过滤机制

📖 **详细文档**: [DeepSeek AI 交易](docs/deepseek/DEEPSEEK_TRADER_README.md)

---

### 🔧 技术亮点
- 🚀 **Rust编写** - 内存安全、零成本抽象、极致性能
- ⚡ **异步架构** - 基于tokio异步运行时
- 🔒 **类型安全** - 编译期保证正确性
- 📊 **实时统计** - 账户状态、持仓盈亏一目了然
- 🌐 **多交易所** - Gate.io / OKX / Binance / Hyperliquid

---

## 📚 文档

| 功能 | 文档 |
|-----|----------|
| **主力资金追踪** | [QUICKSTART_SMART_MONEY.md](QUICKSTART_SMART_MONEY.md) |
| **传统跟单** | [QUICKSTART.md](QUICKSTART.md) |
| **多交易所支持** | [docs/user-guide/README_MULTI_EXCHANGE.md](docs/user-guide/README_MULTI_EXCHANGE.md) |
| **DeepSeek AI** | [docs/deepseek/DEEPSEEK_TRADER_README.md](docs/deepseek/DEEPSEEK_TRADER_README.md) |
| **系统架构** | [docs/technical/SYSTEM_ARCHITECTURE.md](docs/technical/SYSTEM_ARCHITECTURE.md) |
| **Hyperliquid** | [docs/technical/HYPERLIQUID_README.md](docs/technical/HYPERLIQUID_README.md) |

---

## 📦 快速开始

### 前置要求
- Rust 1.70+ ([安装指南](https://www.rust-lang.org/tools/install))
- Binance账户（带单者+跟单者）
- Telegram Bot Token ([创建教程](https://core.telegram.org/bots#6-botfather))

### 1. 克隆项目
```bash
cd /home/hantiv/code/Web3/apps/rust-trading-bot
```

### 2. 配置环境变量
```bash
cp .env.example .env
nano .env
```

编辑配置：
```env
# Telegram Bot配置
TELOXIDE_TOKEN=123456:ABCdefGHIjklMNOpqrsTUVwxyz  # 你的Bot Token

# 跟单者（你的账户）
BINANCE_API_KEY=your_api_key
BINANCE_SECRET_KEY=your_secret_key

# 带单者（只需只读权限）
LEADER_API_KEY=leader_api_key
LEADER_SECRET_KEY=leader_secret_key

# 跟单参数
COPY_RATIO=0.5          # 50%资金跟单
MAX_POSITION_SIZE=100   # 单笔最大100 USDT
LEVERAGE=3              # 3倍杠杆
BINANCE_TESTNET=true    # 测试网（改为false启用主网）
```

### 3. 编译运行
```bash
# 开发模式（带详细日志）
cargo run

# 生产模式（优化编译）
cargo build --release
./target/release/rust-trading-bot
```

---

## 🤖 Telegram Bot 命令

启动Bot后，在Telegram中发送以下命令：

| 命令 | 说明 |
|------|------|
| `/help` | 显示帮助信息 |
| `/start` | ▶️ 启动自动跟单 |
| `/stop` | ⏹️ 停止跟单 |
| `/status` | 📊 查看账户状态 |
| `/positions` | 📦 查看当前持仓 |
| `/stats` | 📈 查看跟单统计 |
| `/ratio 0.3` | ⚙️ 设置跟单比例为30% |

---

## 🏗️ 架构设计

```
rust-trading-bot/
├── src/
│   ├── main.rs              # 主程序入口
│   ├── binance_client.rs    # Binance API封装
│   ├── copy_trader.rs       # 跟单核心逻辑
│   └── telegram_bot.rs      # Telegram Bot接口
├── Cargo.toml               # 项目依赖
└── .env                     # 配置文件
```

### 核心模块

#### 1. BinanceClient - API封装
```rust
// 开多仓
client.open_long("BTCUSDT", 0.01, 5).await?;

// 开空仓
client.open_short("ETHUSDT", 0.1, 3).await?;

// 平仓
client.close_position("BTCUSDT", "LONG", 0.01).await?;
```

#### 2. CopyTrader - 跟单引擎
```rust
// 创建跟单器
let copy_trader = CopyTrader::new(leader_client, follower_client, config);

// 启动监控（自动跟单）
copy_trader.start_monitoring().await?;
```

#### 3. TelegramBot - 远程控制
```rust
// 创建Bot
let bot = TelegramBot::new(token, copy_trader);

// 运行Bot（阻塞）
bot.run().await;
```

---

## 🛡️ 风险控制

### 内置风险管理
- ✅ **最大仓位限制** - 单笔不超过配置金额
- ✅ **自动止损** - 可配置止损百分比（默认5%）
- ✅ **杠杆限制** - 防止过度杠杆
- ✅ **测试网支持** - 先在测试网验证策略

### 风险提示 ⚠️
```
❗ 合约交易有极高风险，可能导致本金全部损失
❗ 杠杆交易会放大盈利和亏损
❗ 请先在测试网充分测试
❗ 只投入你能承受损失的资金
❗ 本软件仅供学习交流，使用责任自负
```

---

## 📈 跟单逻辑

### 开仓跟单
```
1. 检测到带单者开仓 →
2. 计算跟单数量 = 带单者仓位 × 跟单比例 →
3. 限制最大仓位 →
4. 执行开仓 →
5. 设置止损（可选）
```

### 平仓跟单
```
1. 检测到带单者平仓 →
2. 查找自己对应持仓 →
3. 全部平仓 →
4. 记录盈亏
```

---

## 🔧 高级配置

### 自定义跟单策略

修改 `src/copy_trader.rs` 中的逻辑：

```rust
// 自定义仓位计算
fn calculate_copy_quantity(&self, leader_pos: &Position) -> Result<f64> {
    // 你的策略逻辑
    let quantity = /* 计算逻辑 */;
    Ok(quantity)
}

// 自定义止损价格
fn calculate_stop_loss_price(&self, pos: &Position) -> f64 {
    // 你的止损逻辑
    pos.entry_price * 0.95  // 5%止损
}
```

---

## 📊 性能指标

| 指标 | 数值 |
|------|------|
| 响应延迟 | < 100ms |
| 内存占用 | ~10MB |
| CPU占用 | < 1% |
| 检测频率 | 5秒/次 |

---

## 🐛 故障排查

### 常见问题

**Q: Bot无法启动？**
```bash
# 检查Rust版本
rustc --version  # 应该 >= 1.70

# 查看详细错误
RUST_LOG=debug cargo run
```

**Q: API连接失败？**
- 检查API Key是否正确
- 检查IP白名单设置
- 测试网地址是否正确

**Q: 跟单没有执行？**
- 检查带单者是否有持仓变化
- 查看日志输出 `RUST_LOG=info`
- 确认跟单已启动 `/start`

---

## 📝 开发计划

- [ ] WebSocket实时订阅（降低延迟）
- [ ] 更多交易对支持
- [ ] Web Dashboard
- [ ] 回测功能
- [ ] 策略回放

---

## 🤝 贡献指南

欢迎提交Issue和PR！

```bash
# Fork项目
git fork https://github.com/yourusername/rust-trading-bot

# 创建特性分支
git checkout -b feature/awesome-feature

# 提交更改
git commit -m "Add awesome feature"

# 推送到分支
git push origin feature/awesome-feature
```

---

## 📄 许可证

MIT License - 详见 [LICENSE](LICENSE)

---

## 🙏 致谢

- [teloxide](https://github.com/teloxide/teloxide) - 优秀的Telegram Bot框架
- [binance-rs-async](https://github.com/wisespace-io/binance-rs) - Binance API Rust实现
- Rust社区的所有贡献者

---

<div align="center">

**⚠️ 风险提示：合约交易有风险，投资需谨慎 ⚠️**

Made with ❤️ by Rust

</div>