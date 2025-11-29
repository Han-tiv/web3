# 主力资金追踪交易系统 - 快速启动指南

## 🎯 系统概述

基于**主力资金流向信号** + **1小时K线技术分析**的短期/日内交易系统。

核心优势：
- ✅ 识别主力资金关键位置（最大成交量K线）
- ✅ 动态支撑阻力位计算
- ✅ 智能信号优先级评估
- ✅ 自动止损止盈设置

---

## 📋 前置要求

### 环境变量配置

```bash
# 必需配置（根据使用的交易所选择）
export GATE_API_KEY="your_gate_api_key"
export GATE_SECRET="your_gate_secret"

# 或者使用 OKX
export OKX_API_KEY="your_okx_api_key"
export OKX_SECRET="your_okx_secret"
export OKX_PASSWORD="your_okx_password"

# 或者使用 Binance
export BINANCE_API_KEY="your_binance_api_key"
export BINANCE_SECRET="your_binance_secret"
```

---

## 🚀 快速启动

### 1. 编译项目

```bash
cd /home/hanins/code/web3/apps/rust-trading-bot
cargo build --release --bin smart_money_trader
```

### 2. 运行交易系统（演示模式）

```bash
# 设置日志级别
export RUST_LOG=info

# 运行
cargo run --bin smart_money_trader
```

---

## 📊 系统工作流程

```
1️⃣ 接收主力资金信号
   ↓
2️⃣ 拉取最近48根1h K线
   ↓
3️⃣ 找到最大成交量K线
   → 大阳线：阻力位 = 最高价
   → 大阴线：支撑位 = 最低价
   ↓
4️⃣ 技术指标分析
   - RSI（超买超卖）
   - MACD（趋势确认）
   - 布林带（波动区间）
   ↓
5️⃣ 生成交易信号
   - 突破做多
   - 回踩做多
   - 破位做空
   - 平仓离场
   ↓
6️⃣ 执行交易（实盘模式）
```

---

## 🎮 信号类型说明

### 1. 突破做多 (LongBreakout)

**触发条件**：
- 资金流入 + 价格突破主力阻力位
- 成交量放大 > 1.5倍平均成交量

**示例**：
```
主力阻力位: $69,500
当前价格: $69,600 (突破)
成交量: 2.1倍平均值
RSI: 58 (中性)
→ 信号: 高优先级突破做多
```

---

### 2. 回踩做多 (LongPullback)

**触发条件**：
- 资金流入 + 价格接近主力支撑位
- RSI < 40（超卖）
- 支撑位强度 > 60分

**示例**：
```
主力支撑位: $68,000
当前价格: $68,200 (接近支撑)
RSI: 35 (超卖)
→ 信号: 中等优先级回踩做多
```

---

### 3. 破位做空 (ShortBreakdown)

**触发条件**：
- 资金流出 + 价格跌破主力支撑位
- RSI < 35

**示例**：
```
主力支撑位: $68,000
当前价格: $67,850 (跌破)
RSI: 32
→ 信号: 高优先级破位做空
```

---

### 4. 平仓离场 (ClosePosition)

**触发条件**：
- 持有多单 + 主力资金流出
- 价格触及止盈/止损位

---

## 🔧 配置参数

编辑 `src/bin/smart_money_trader.rs` 中的配置：

```rust
TradingConfig {
    symbol: "BTC/USDT",         // 交易对
    timeframe: "1h",            // K线周期（1小时）
    leverage: 5,                // 杠杆倍数
    exchange: ExchangeType::Gate,
    base_position_usdt: 50.0,   // 基础仓位
    max_position_usdt: 200.0,   // 最大仓位
}
```

---

## 📡 接入真实主力资金信号

### 方法1：Telegram Bot（推荐）

```rust
// TODO: 实现 Telegram 监听器
// 参考：src/telegram_bot.rs

// 伪代码
async fn listen_telegram_signals() -> MoneyFlowSignal {
    // 监听特定频道/群组
    // 解析主力资金流向消息
    // 返回标准化信号
}
```

### 方法2：Webhook API

```rust
// 启动 HTTP 服务器接收信号
use axum::{Router, Json};

#[derive(Deserialize)]
struct SignalPayload {
    direction: String,  // "inflow" / "outflow"
    strength: f64,
    timestamp: i64,
}

async fn receive_signal(
    Json(payload): Json<SignalPayload>
) -> String {
    // 处理信号
    // 触发交易分析
    "OK".to_string()
}
```

### 方法3：手动触发

```rust
// 在代码中直接创建信号
let signal = MoneyFlowSignal {
    timestamp: Utc::now().timestamp(),
    direction: MoneyFlowDirection::Inflow,
    strength: 0.8,
    source: "Manual".to_string(),
    symbol: "BTC/USDT".to_string(),
    note: Some("观察到大额流入".to_string()),
};
```

---

## 🛡️ 风险控制

### 自动止损止盈

系统自动计算：

```rust
// 突破做多
stop_loss = 最近支撑位 * 0.98     // 支撑下方2%
take_profit = 入场价 * 1.05       // 5%目标

// 回踩做多
stop_loss = 支撑位 * 0.98
take_profit = 最近阻力位 * 0.99   // 阻力下方1%
```

### 仓位管理

根据信号优先级动态调整：

| 优先级 | 倍数 | 示例仓位 |
|-------|------|---------|
| Critical | 1.5x | 75 USDT |
| High | 1.2x | 60 USDT |
| Medium | 1.0x | 50 USDT |
| Low | 0.6x | 30 USDT |

---

## 📈 性能监控

### 查看关键位识别

运行后会输出：

```
🔍 最大成交量K线: index=23, volume=5234.50, open=68000, close=69500
【关键价格位】
1. 阻力 $69500.00 | 强度:80% | 测试:1次
2. 支撑 $68000.00 | 强度:70% | 测试:1次
3. 阻力 $70200.00 | 强度:60% | 测试:2次
4. 支撑 $67500.00 | 强度:60% | 测试:1次
```

### 查看交易信号

```
【交易信号】
类型: LongBreakout
优先级: High
入场价: $69600.00
止损价: $68300.00
止盈价: $73080.00
置信度: 82.5%
理由: 突破阻力位 $69500.00, 资金流入强度:0.75, 成交量:2.1倍
```

---

## 🧪 测试模式

当前版本默认运行在**演示模式**，不会执行真实交易。

### 启用实盘交易

在 `src/bin/smart_money_trader.rs` 中：

```rust
// 找到这一行并取消注释
// execute_trade(exchange, &signal, config).await?;
```

**⚠️ 警告**：启用实盘前请：
1. 确认已充分测试
2. 从小仓位开始
3. 设置合理的最大仓位限制

---

## 📝 日志说明

```bash
# 详细日志
export RUST_LOG=debug
cargo run --bin smart_money_trader

# 仅重要信息
export RUST_LOG=info
cargo run --bin smart_money_trader

# 仅错误
export RUST_LOG=error
cargo run --bin smart_money_trader
```

---

## 🔍 故障排查

### 问题1：获取K线失败

```
❌ 获取K线失败: API rate limit
```

**解决**：
- 检查 API 密钥权限
- 等待 API 限流解除
- 降低请求频率

### 问题2：无法识别关键位

```
⚠️  K线数据不足 (需要至少24根)
```

**解决**：
- 等待积累足够的K线数据
- 或降低 `lookback_hours` 参数

### 问题3：信号强度不足

```
⚠️  资金流向强度不足: 0.45 < 0.60
```

**解决**：
- 降低 `min_money_flow_strength` 阈值
- 或等待更强的资金信号

---

## 📚 核心模块文档

| 模块 | 文件 | 功能 |
|-----|------|------|
| 关键位识别 | `src/key_level_finder.rs` | 找最大成交量K线，识别支撑阻力 |
| 主力追踪 | `src/smart_money_tracker.rs` | 资金流向分析，生成交易信号 |
| 技术分析 | `src/technical_analysis.rs` | RSI/MACD/布林带计算 |
| 交易执行 | `src/bin/smart_money_trader.rs` | 主程序，交易循环 |

---

## 🎓 策略核心理念

> **跟随主力资金，在关键位置建仓**

1. **主力建仓区** = 最大成交量K线的价格区间
2. **支撑位** = 主力不愿跌破的防守线
3. **阻力位** = 主力可能出货的压力区
4. **突破确认** = 成交量放大 + 价格突破
5. **止损原则** = 关键位下方，避免被主力洗盘

---

## 📞 下一步

1. **集成真实信号源**：Telegram Bot / Webhook
2. **回测框架**：验证策略有效性
3. **实时监控面板**：可视化关键位和信号
4. **多币种支持**：扩展到 ETH, SOL 等
5. **高级仓位管理**：马丁格尔/网格策略

---

## ✅ 检查清单

启动前确认：

- [ ] 环境变量已配置
- [ ] 交易所 API 权限正确
- [ ] 账户有足够余额
- [ ] 理解各个信号类型
- [ ] 知道如何手动平仓
- [ ] 设置了合理的仓位限制

---

**准备好了？开始追踪主力资金！** 🚀
