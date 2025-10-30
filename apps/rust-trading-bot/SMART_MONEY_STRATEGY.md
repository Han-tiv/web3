# 主力资金追踪交易策略设计

## 策略概述

基于**主力资金流向信号**，结合**1小时K线技术分析**，识别主力资金关键位置，进行短期和日内交易。

---

## 核心思路

### 1️⃣ 输入信号
- **外部主力资金流向通知**（Telegram/Webhook）
  - 资金流入/流出方向
  - 流入强度
  - 时间戳

### 2️⃣ K线分析（1小时周期）
- 最近N根K线中找**最大成交量K线**
- 该K线的**最高价/最低价**作为主力关键位

### 3️⃣ 主力位识别逻辑

```
主力资金流入 + 大成交量K线：
  → 在大阳线最高价设置阻力位
  → 在大阴线最低价设置支撑位

主力资金流出 + 大成交量K线：
  → 在大阴线最低价设置破位点
  → 观察是否出现新支撑
```

---

## 策略架构

### 模块设计

```
rust-trading-bot/
├── src/
│   ├── smart_money_tracker.rs      # 🆕 主力资金追踪器
│   ├── key_level_finder.rs         # 🆕 关键位识别
│   ├── intraday_signal_engine.rs   # 🆕 日内信号引擎
│   ├── telegram_signal_listener.rs # 🆕 信号监听器
│   └── bin/
│       └── smart_money_trader.rs   # 🆕 主力追踪交易器
```

---

## 关键位识别算法

### 算法1：最大成交量K线法

```rust
// 找最近24根1h K线（24小时）中成交量最大的K线
fn find_max_volume_kline(klines: &[Kline], lookback: usize) -> &Kline {
    klines.iter()
        .rev()
        .take(lookback)
        .max_by(|a, b| a.volume.partial_cmp(&b.volume).unwrap())
        .unwrap()
}

// 根据K线方向确定关键位
fn determine_key_level(kline: &Kline, money_flow: MoneyFlow) -> KeyLevel {
    match money_flow {
        MoneyFlow::Inflow => {
            if kline.close > kline.open {
                // 大阳线 + 资金流入 = 阻力位
                KeyLevel::Resistance(kline.high)
            } else {
                // 大阴线 + 资金流入 = 支撑位（主力在低位吸筹）
                KeyLevel::Support(kline.low)
            }
        },
        MoneyFlow::Outflow => {
            // 资金流出 = 警戒位
            KeyLevel::Warning(kline.low)
        }
    }
}
```

---

### 算法2：支撑阻力位强度评分

```rust
struct KeyLevelScore {
    price: f64,
    strength: f64,  // 0-100分
    test_count: u32, // 被测试次数
    volume_support: f64, // 成交量支撑度
}

// 评分标准
fn score_key_level(level: f64, klines: &[Kline]) -> f64 {
    let mut score = 0.0;
    
    // 1. 被测试次数（每次+15分）
    let tests = count_price_tests(level, klines);
    score += tests as f64 * 15.0;
    
    // 2. 成交量集中度（最高+30分）
    let volume_concentration = calc_volume_at_level(level, klines);
    score += volume_concentration * 30.0;
    
    // 3. K线形态确认（反转形态+25分）
    if has_reversal_pattern(level, klines) {
        score += 25.0;
    }
    
    // 4. 时间新鲜度（越近越高，最高+30分）
    let recency = calc_recency_score(level, klines);
    score += recency * 30.0;
    
    score.min(100.0)
}
```

---

## 交易信号生成

### 信号类型

| 信号 | 条件 | 操作 |
|-----|------|------|
| **强势做多** | 资金流入 + 价格突破主力阻力位 + 成交量放大 | 开多仓 |
| **回踩支撑做多** | 资金流入 + 价格回踩主力支撑位 + RSI<40 | 开多仓 |
| **破位止损** | 价格跌破主力支撑位 + 成交量异常 | 平多/开空 |
| **阻力位止盈** | 价格触及主力阻力位 + RSI>70 | 平多 |

---

### 信号优先级

```rust
enum SignalPriority {
    Critical,   // 立即执行（破位）
    High,       // 高优先级（突破）
    Medium,     // 中等（回踩）
    Low,        // 低优先级（观察）
}

// 信号评分
fn calculate_signal_priority(signal: &Signal) -> SignalPriority {
    let score = 0;
    
    // 资金流向强度
    score += signal.money_flow_strength * 40;
    
    // 关键位强度
    score += signal.key_level_score * 30;
    
    // 技术指标确认度
    score += signal.tech_confirmation * 30;
    
    match score {
        80..=100 => SignalPriority::Critical,
        60..=79 => SignalPriority::High,
        40..=59 => SignalPriority::Medium,
        _ => SignalPriority::Low,
    }
}
```

---

## 风险控制

### 1. 仓位管理

```rust
struct SmartMoneyPositionManager {
    base_position: f64,          // 基础仓位
    max_position: f64,           // 最大仓位
    scaling_factor: f64,         // 加仓系数
}

impl SmartMoneyPositionManager {
    fn calculate_position(&self, signal: &Signal) -> f64 {
        let base = self.base_position;
        
        // 根据信号优先级调整
        let multiplier = match signal.priority {
            SignalPriority::Critical => 1.5,
            SignalPriority::High => 1.2,
            SignalPriority::Medium => 1.0,
            SignalPriority::Low => 0.5,
        };
        
        // 根据主力资金强度调整
        let money_flow_adj = 1.0 + (signal.money_flow_strength * 0.3);
        
        let position = base * multiplier * money_flow_adj;
        position.min(self.max_position)
    }
}
```

### 2. 止损策略

```rust
// 动态止损位
fn calculate_stop_loss(entry_price: f64, key_level: &KeyLevel) -> f64 {
    match key_level {
        KeyLevel::Support(level) => {
            // 支撑位下方1-2%
            level * 0.98
        },
        KeyLevel::Resistance(level) => {
            // 阻力位上方设置
            level * 1.02
        },
        _ => entry_price * 0.97 // 默认3%止损
    }
}
```

---

## 数据结构定义

```rust
// 主力资金信号
#[derive(Debug, Clone)]
pub struct MoneyFlowSignal {
    pub timestamp: i64,
    pub direction: MoneyFlowDirection,  // Inflow/Outflow
    pub strength: f64,                  // 0.0-1.0
    pub source: String,                 // 信号来源
    pub symbol: String,
}

#[derive(Debug, Clone)]
pub enum MoneyFlowDirection {
    Inflow,   // 流入
    Outflow,  // 流出
    Neutral,  // 中性
}

// 关键价格位
#[derive(Debug, Clone)]
pub struct KeyLevel {
    pub price: f64,
    pub level_type: LevelType,
    pub strength: f64,
    pub volume: f64,
    pub last_test_time: i64,
    pub test_count: u32,
}

#[derive(Debug, Clone)]
pub enum LevelType {
    Support,      // 支撑位
    Resistance,   // 阻力位
    Warning,      // 警戒位
}

// 交易信号
#[derive(Debug, Clone)]
pub struct TradingSignal {
    pub signal_type: SignalType,
    pub entry_price: f64,
    pub stop_loss: f64,
    pub take_profit: f64,
    pub position_size: f64,
    pub priority: SignalPriority,
    pub reason: String,
    pub key_levels: Vec<KeyLevel>,
}

#[derive(Debug, Clone)]
pub enum SignalType {
    LongBreakout,       // 突破做多
    LongPullback,       // 回踩做多
    ShortBreakdown,     // 破位做空
    ClosePosition,      // 平仓
    Hold,               // 持有
}
```

---

## 实现步骤

### Phase 1: 核心模块（1-2天）
- [x] ~~删除 ds 目录~~
- [ ] 实现 `key_level_finder.rs` - 关键位识别
- [ ] 实现 `smart_money_tracker.rs` - 主力追踪
- [ ] 扩展 `technical_analysis.rs` - 添加成交量分析

### Phase 2: 信号引擎（2-3天）
- [ ] 实现 `intraday_signal_engine.rs` - 信号生成
- [ ] 集成 Telegram 信号监听
- [ ] 实现仓位管理器

### Phase 3: 交易执行器（2天）
- [ ] 实现 `smart_money_trader.rs` - 主程序
- [ ] 集成到现有交易所接口
- [ ] 回测框架

### Phase 4: 优化测试（1-2天）
- [ ] 参数优化
- [ ] 实盘小仓位测试
- [ ] 性能监控

---

## 示例交易流程

```
1. 收到主力资金流入信号（Telegram）
   ↓
2. 拉取最近24根1h K线
   ↓
3. 找到最大成交量K线
   - 成交量: 5000 BTC
   - 大阳线: $68,000 → $69,500
   ↓
4. 确定关键位
   - 阻力位: $69,500 (强度: 85/100)
   - 支撑位: $68,000 (强度: 78/100)
   ↓
5. 当前价格: $68,800
   ↓
6. 生成信号:
   - 类型: LongPullback（回踩做多）
   - 入场: $68,200（靠近支撑）
   - 止损: $67,800（支撑下方）
   - 止盈: $69,400（阻力位）
   - 仓位: 0.05 BTC
   ↓
7. 等待价格回踩 → 触发入场
   ↓
8. 执行交易
```

---

## 配置示例

```toml
[smart_money_strategy]
# K线设置
timeframe = "1h"
lookback_hours = 24

# 关键位识别
key_level_score_threshold = 60.0
max_key_levels = 5

# 仓位管理
base_position_usdt = 50.0
max_position_usdt = 200.0
risk_per_trade = 0.02  # 2%

# 信号过滤
min_money_flow_strength = 0.6
min_volume_ratio = 1.5

# 交易限制
max_trades_per_day = 5
min_trade_interval_minutes = 60
```

---

## 优势分析

### ✅ 相比纯技术指标策略
- 结合真实资金流向，信号更准确
- 识别主力操作意图
- 减少虚假突破

### ✅ 相比传统支撑阻力
- 动态识别，跟随市场变化
- 成交量验证，更可靠
- 时效性强，适合短期交易

### ✅ 风险管理
- 动态止损止盈
- 仓位智能调整
- 多重确认机制

---

## 下一步行动

1. **立即开始**：实现 `key_level_finder.rs` 模块
2. **集成测试**：使用历史数据验证算法
3. **实盘小仓位测试**：验证策略有效性

准备好开始实现吗？
