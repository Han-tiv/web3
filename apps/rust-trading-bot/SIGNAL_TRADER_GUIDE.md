# 📡 Telegram 信号自动交易系统

## ⚠️ 重要警告

- 🔴 **真实交易有风险，可能导致本金全部损失**
- 🔴 **15x 杠杆仍具高风险，请谨慎操作**
- 🔴 **默认交易功能已禁用，仅监听模式**

## 🎯 功能说明

监听 Telegram 频道 **CM AI SIGNAL** (ID: 2291145819) 的交易信号：

### 开仓信号
```
B2USDT - 看跌📉        → 开空仓
ETHUSDT - 看涨📈       → 开多仓
```

### 平仓信号
```
SUPERUSDT - 看跌跟踪结束  → 平掉对应空仓
XANUSDT - 看涨跟踪结束    → 平掉对应多仓
```

## ⚙️ 配置说明

### 当前配置 (`/home/hanins/code/.env`)

```bash
# Binance 配置
BINANCE_API_KEY=...
BINANCE_SECRET_KEY=...
BINANCE_TESTNET=false        # ✅ 主网

# Telegram 配置
TELEGRAM_API_ID=13741857
TELEGRAM_API_HASH=...
TELEGRAM_PHONE=+18489994567
TARGET_CHANNEL_ID=2291145819

# 交易配置
SIGNAL_LEVERAGE=15                     # 15x 杠杆
SIGNAL_MARGIN=2                        # 2 USDT 保证金
SIGNAL_MARGIN_TYPE=ISOLATED            # 逐仓模式
SIGNAL_MULTI_ASSET_MODE=SINGLE         # 单币种保证金
SIGNAL_TRADING_ENABLED=false           # ❌ 默认禁用交易
```

## 🚀 使用方法

### 1. 仅监听模式（推荐先测试）

```bash
cd /home/hanins/code/apps/rust-trading-bot
./target/release/signal_trader
```

程序会：
- ✅ 监听频道消息
- ✅ 解析交易信号
- ✅ 显示信号内容
- ❌ **不执行真实交易**

### 2. 启用真实交易（谨慎！）

**步骤 1**: 修改配置
```bash
nano /home/hanins/code/.env

# 修改这一行:
SIGNAL_TRADING_ENABLED=true    # 启用交易
```

**步骤 2**: 运行程序
```bash
./target/release/signal_trader
```

程序会：
- ✅ 监听频道
- ✅ 解析信号
- ✅ **自动执行交易**（开仓/平仓）

## 📊 交易逻辑

### 开仓计算
```
当前价格 = 查询 Binance
数量 = (保证金 × 杠杆) / 当前价格
     = (2 USDT × 15) / 价格
```

示例：
- ETH 价格 2400 USDT
- 数量 = 15 / 2400 = 0.00625 ETH
- 名义价值 = 30 USDT

### 平仓逻辑
- 查询当前持仓
- 找到对应交易对
- 全部平仓

## 🛡️ 风险控制

1. **小仓位测试**
   - 当前配置：2 USDT × 15x = 30 USDT 名义
   - 可根据交易对最小名义价值适度调整（例如 1~3 USDT 保证金）

2. **监控余额**
   ```bash
   ./check_balance.sh   # 随时查看账户状态
   ```

3. **手动止损**
   - 登录 Binance App
   - 随时手动平仓

## 📝 日志查看

详细日志模式：
```bash
RUST_LOG=info ./target/release/signal_trader
```

## 🔧 故障排查

### 问题：交易未执行
- 检查 `SIGNAL_TRADING_ENABLED=true`
- 查看日志错误信息
- 验证 API Key 权限

### 问题：无法连接 Telegram
- 检查网络连接
- 验证 session.session 文件存在
- 重新登录（删除 session.session）

### 问题：Binance API 错误
- 检查 IP 白名单
- 验证 API Key 权限（需要期货交易权限）
- 确认余额充足

## ⚡ 快速命令

```bash
# 仅监听（安全）
./target/release/signal_trader

# 查看余额
./check_balance.sh

# 停止程序
Ctrl+C

# 启用交易（修改配置后）
nano /home/hanins/code/.env    # SIGNAL_TRADING_ENABLED=true
./target/release/signal_trader
```

## 📌 注意事项

1. **首次运行** 需要输入 Telegram 验证码
2. **session.session** 会保存登录状态
3. **主网交易** 消耗真实资金
4. **15x 杠杆** 波动 6.7% 左右即可触发爆仓
5. **建议先用小仓位** 测试信号准确性

## 🆘 紧急停止

如果程序失控：
1. `Ctrl+C` 停止程序
2. 登录 Binance App 手动平仓
3. 设置 `SIGNAL_TRADING_ENABLED=false`

---

**风险自负，谨慎操作！**
