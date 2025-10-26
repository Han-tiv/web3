# 🏗️ 系统架构文档

**项目**: Rust Trading Bot  
**版本**: v2.0  
**更新**: 2025-10-26

---

## 📊 系统概览

```
┌─────────────────────────────────────────────────────────────┐
│                   Rust Trading Bot v2.0                     │
│                     多平台交易系统                           │
└─────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
    ┌───▼────┐           ┌───▼────┐           ┌───▼────┐
    │  CEX   │           │  DEX   │           │  链钱包 │
    │  5个   │           │  1个   │           │  1个   │
    └───┬────┘           └───┬────┘           └───┬────┘
        │                    │                     │
  ┌─────┴──────┐       ┌────▼─────┐         ┌────▼────┐
  │  Binance   │       │Hyperliquid│        │ Solana  │
  │   OKX      │       └──────────┘         └─────────┘
  │  Bitget    │
  │   Bybit    │
  │  Gate.io   │
  └────────────┘
```

**总计**: 7 个平台 | 619.44 USDT | 0 持仓

---

## 🧩 核心模块

### 1. Exchange Trait (交易所抽象)
**文件**: `src/exchange_trait.rs`

```rust
pub trait ExchangeClient: Send + Sync {
    fn get_exchange_name(&self) -> &str;
    async fn get_positions(&self) -> Result<Vec<Position>>;
    async fn get_account_info(&self) -> Result<AccountInfo>;
    async fn get_current_price(&self, symbol: &str) -> Result<f64>;
    async fn open_long(&self, symbol: &str, size: f64, leverage: u32) -> Result<()>;
    async fn open_short(&self, symbol: &str, size: f64, leverage: u32) -> Result<()>;
    async fn close_position(&self, symbol: &str) -> Result<()>;
}
```

**设计理念**: 统一接口，多态实现

---

### 2. 交易所客户端实现

#### 2.1 Binance Client
**文件**: `src/binance_client.rs`

**账户查询架构**:
```
BinanceClient::get_account_info()
│
├─> 1. 统一账户 (PAPI)
│   └─> /papi/v1/balance
│       ├─> U本位合约: 30.64 USDT
│       ├─> 币本位合约: 0 USDT
│       └─> 杠杆账户: 0 USDT
│
├─> 2. 现货账户 (Spot)
│   └─> /api/v3/account
│       └─> USDT/USDC: 0.0000 USDT
│
└─> 3. 资金账户 (Funding) ⭐
    └─> /sapi/v1/asset/get-funding-asset
        ├─> USDC: 127.84
        ├─> USDT: 50.32
        └─> DOGE: 0.001
```

**特点**:
- ✅ 支持完整交易功能
- ✅ HMAC SHA256 签名
- ✅ IPv4 专用连接
- ✅ 资金账户查询优化

#### 2.2 其他 CEX 客户端
| 客户端 | 文件 | 特点 |
|--------|------|------|
| **OKX** | `okx_client.rs` | API Key + Passphrase |
| **Bitget** | `bitget_client.rs` | 合约账户 |
| **Bybit** | `bybit_client.rs` | UNIFIED 账户 |
| **Gate.io** | `gate_client.rs` | 合约+现货双账户 |

#### 2.3 DEX 客户端
**Hyperliquid**  
**文件**: `hyperliquid_client.rs`

**特点**:
- ✅ EIP-712 签名
- ✅ 以太坊私钥管理
- ✅ 完整交易支持
- ⚠️ 持仓解析待优化

#### 2.4 链钱包客户端

**BSC Wallet**  
**文件**: `bsc_wallet.rs`
- ✅ BNB 余额查询
- ✅ ERC20 代币 (USDT, BUSD)
- ✅ 实时价格集成

**Solana Wallet**  
**文件**: `solana_wallet.rs`
- ✅ SOL 余额查询
- ✅ HTTP RPC 调用
- ✅ 实时价格集成

---

### 3. 价格服务
**文件**: `src/price_service.rs`

```
PriceService
│
├─> API: CoinGecko (免费)
├─> 缓存: 60 秒
├─> 支持: BTC, ETH, BNB, SOL, USDT, USDC...
│
└─> 自动失败回退
    └─> 默认价格
```

**特点**:
- ✅ 60秒缓存机制
- ✅ 批量查询支持
- ✅ 自动失败回退
- ✅ 符号自动映射

---

### 4. 多交易所执行器
**文件**: `src/multi_exchange_executor.rs`

```
MultiExchangeExecutor
│
├─> 管理所有交易所客户端
├─> 并发查询资产
├─> 统一交易接口
│
└─> 支持:
    ├─> 资产汇总
    ├─> 持仓管理
    └─> 批量交易
```

---

## 🛠️ 工具程序

### 核心程序

#### 1. show_assets
**文件**: `src/bin/show_assets.rs`

**功能**: 查看所有平台资产
```
cargo run --release --bin show_assets
```

**输出**:
- 各平台账户余额
- 持仓信息
- 总资产汇总

#### 2. multi_signal_trader
**文件**: `src/bin/multi_signal_trader.rs`

**功能**: 多信号源交易机器人
- ✅ Telegram 信号监听
- ✅ 多平台同时交易
- ✅ 风险管理

#### 3. signal_trader
**文件**: `src/bin/signal_trader.rs`

**功能**: 单信号源交易机器人
- ✅ Hemi 信号解析
- ✅ 自动开平仓
- ✅ 杠杆管理

### 辅助工具

| 工具 | 文件 | 功能 |
|------|------|------|
| **胜率分析** | `analyze_win_rate.rs` | 统计交易胜率 |
| **余额检查** | `check_balance.rs` | 快速余额查询 |
| **利润监控** | `profit_monitor.rs` | 实时利润追踪 |
| **频道管理** | `get_channels.rs` | 获取 Telegram 频道 |
| **频道列表** | `list_channels.rs` | 列出所有频道 |
| **频道监控** | `monitor_channel.rs` | 监控频道消息 |

---

## 📦 依赖管理

### 核心依赖
```toml
[dependencies]
# 异步运行时
tokio = { version = "1.37", features = ["full"] }

# HTTP 客户端
reqwest = { version = "0.12", features = ["json", "cookies"] }

# JSON 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 以太坊签名
ethers = { version = "2.0", features = ["abigen", "ws"] }
secp256k1 = { version = "0.28", features = ["recovery"] }

# Telegram
teloxide = { version = "0.12", features = ["macros"] }

# 日志
log = "0.4"
env_logger = "0.11"
```

### 区块链支持
- ✅ Ethereum / BSC: `ethers`
- ✅ Solana: HTTP RPC (无额外依赖)

---

## 🔐 安全架构

### 1. API Key 管理
```
.env 文件
├─> API_KEY
├─> SECRET_KEY
├─> PASSPHRASE (OKX)
└─> PRIVATE_KEY (Hyperliquid, BSC, Solana)
```

**安全措施**:
- ✅ `.env` 在 `.gitignore`
- ✅ 文件权限 600
- ✅ 仅本地存储
- ✅ 不记录日志

### 2. 签名机制

| 交易所 | 签名方式 |
|--------|----------|
| Binance | HMAC SHA256 |
| OKX | HMAC SHA256 + Base64 |
| Bitget | HMAC SHA256 |
| Bybit | HMAC SHA256 |
| Gate.io | HMAC SHA512 |
| Hyperliquid | EIP-712 |

---

## 📊 数据流

```
用户请求
    │
    ▼
┌─────────────┐
│  程序入口   │ (main.rs / bin/*.rs)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ExchangeClient│ (trait 统一接口)
└──────┬──────┘
       │
   ┌───┴───┬───────┬─────────┐
   │       │       │         │
   ▼       ▼       ▼         ▼
┌──────┐┌──────┐┌──────┐┌──────┐
│Binance││ OKX ││Hyper││Solana│
│Client││Client││liquid││Wallet│
└───┬──┘└───┬──┘└───┬──┘└───┬──┘
    │       │       │       │
    ▼       ▼       ▼       ▼
┌───────────────────────────────┐
│      HTTP/RPC 请求             │
└───────────────────────────────┘
    │       │       │       │
    ▼       ▼       ▼       ▼
┌───────────────────────────────┐
│   交易所 / 区块链 API          │
└───────────────────────────────┘
```

---

## 🚀 性能优化

### 1. 并发查询
```rust
// 所有交易所并发查询
let futures: Vec<_> = exchanges
    .iter()
    .map(|ex| ex.get_account_info())
    .collect();

let results = join_all(futures).await;
```

### 2. 价格缓存
- ✅ 60秒缓存
- ✅ 减少 API 调用
- ✅ 提升响应速度

### 3. 连接池复用
- ✅ Reqwest Client 复用
- ✅ Keep-Alive 连接
- ✅ 减少握手开销

---

## 📈 监控与日志

### 日志级别
```bash
RUST_LOG=info  # 生产环境
RUST_LOG=debug # 调试模式
RUST_LOG=warn  # 仅警告
```

### 日志输出
```
[INFO] Binance U本位合约: 30.64 USDT
[INFO] Binance 资金账户 USDC: 127.84
[INFO] ✅ SOL 实时价格: $197.30
[WARN] Binance 资金账户 API 返回错误: 404
```

---

## 🎯 最佳实践

### 1. 错误处理
```rust
// 优雅降级
match api_call().await {
    Ok(data) => process(data),
    Err(e) => {
        warn!("API 调用失败: {}", e);
        use_default_value()
    }
}
```

### 2. 资源管理
```rust
// 使用 Arc 共享客户端
let client = Arc::new(BinanceClient::new(...));
```

### 3. 类型安全
```rust
// 强类型定义
#[derive(Debug, Deserialize)]
struct AccountInfo {
    total_balance: f64,
    available_balance: f64,
    unrealized_pnl: f64,
}
```

---

## 🔮 未来规划

### 短期 (1-2周)
- [ ] 添加更多币种价格查询
- [ ] 优化 Hyperliquid 持仓解析
- [ ] 添加资产变化通知

### 中期 (1-2月)
- [ ] Web Dashboard
- [ ] 历史数据存储
- [ ] 性能监控面板

### 长期 (3-6月)
- [ ] 策略回测系统
- [ ] 机器学习集成
- [ ] 分布式部署

---

## 📚 相关文档

- [多交易所使用指南](./README_MULTI_EXCHANGE.md)
- [Hyperliquid 使用说明](./HYPERLIQUID_README.md)
- [区块链钱包集成](./BLOCKCHAIN_WALLETS.md)
- [优化总结](./OPTIMIZATION_SUMMARY.md)

---

**🏗️ 系统架构完整，运行稳定！** 🚀✨
