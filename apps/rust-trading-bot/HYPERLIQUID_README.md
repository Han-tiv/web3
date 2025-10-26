# 🌊 Hyperliquid 集成说明

## 📋 概述

Hyperliquid 是一个完全去中心化的永续合约交易所，使用以太坊钱包进行签名和交易。

## ✅ 已实现功能

| 功能 | 状态 | 说明 |
|------|------|------|
| 查询账户余额 | ✅ | 支持查询 USDC 账户余额 |
| 查询持仓 | ✅ | 支持查询当前持仓 |
| 查询价格 | ✅ | 支持实时价格查询 |
| 查询交易规则 | ✅ | 支持获取交易对信息 |
| **开仓交易** | ✅ | **EIP-712 签名支持** |
| **平仓交易** | ✅ | **完整交易功能** |

## 🔧 配置说明

### 1. 添加到 .env 文件

```bash
# Hyperliquid 配置（完整交易功能）
HYPERLIQUID_ADDRESS=0x你的以太坊地址
HYPERLIQUID_SECRET=0x你的以太坊私钥
HYPERLIQUID_PROXY_ADDRESS=  # 可选，如使用代理地址
HYPERLIQUID_TESTNET=false  # true=测试网, false=主网
```

### 2. 获取私钥

**⚠️ 重要安全提示**：
- 私钥具有完全的资金控制权，请妥善保管
- 建议使用专门的交易钱包，不要使用存放大量资金的钱包
- 确保 `.env` 文件权限设置为 600（仅所有者可读写）
- 不要将包含私钥的 `.env` 文件提交到 Git

## 📊 使用示例

### 查看账户余额

```bash
cargo run --release --bin show_assets
```

输出示例：
```
┌──────────────────────────────────────────┐
│ 🏦 Hyperliquid                           │
├──────────────────────────────────────────┤
│ 💰 账户余额
│   总余额:              1000.00 USDC
│   可用余额:             950.00 USDC
│   未实现盈亏:             0.00 USDC
│   已用保证金:            50.00 USDC
│
│ 📊 持仓 (1 个)
│   📈 BTCUSDT LONG
│     数量:            0.1000
│     入场价:       42000.00 USDC
│     标记价:       42500.00 USDC
│     盈亏:            50.00 USDC
│     杠杆:               10x
│     保证金:          42.00 USDC
└──────────────────────────────────────────┘
```

## ⚠️ 重要提示

### 交易功能限制

Hyperliquid 的交易需要**以太坊私钥签名**，出于安全考虑，当前实现**仅支持查询功能**。

如果尝试执行交易，会收到如下提示：
```
⚠️  Hyperliquid 需要私钥签名，当前实现仅支持查询功能
开多订单: BTCUSDT 数量: 0.1 杠杆: 10x
错误: Hyperliquid 交易功能需要私钥签名支持，请使用 Web 界面或官方客户端
```

### 交易方式

如需交易，请使用：
1. **Hyperliquid Web 界面**: https://app.hyperliquid.xyz
2. **官方 Python SDK**: https://github.com/hyperliquid-dex/hyperliquid-python-sdk
3. **MetaMask 等钱包**: 直接连接 Hyperliquid DApp

## 🔐 安全性

### 为什么不支持交易？

1. **私钥安全**: Hyperliquid 需要以太坊私钥签名交易
2. **避免风险**: 将私钥存储在配置文件中存在极大安全风险
3. **最佳实践**: 交易应该在受信任的钱包或官方客户端中进行

### 推荐的多交易所跟单方案

如果你想在 Hyperliquid 上跟单：
- ✅ 使用本系统监控信号
- ✅ 在其他中心化交易所（Binance, OKX 等）自动交易
- ✅ 在 Hyperliquid 上手动执行（通过 Web 界面）

## 🌐 Hyperliquid 特点

### 优势
- ✅ 完全去中心化
- ✅ 无需 KYC
- ✅ 低交易费用
- ✅ 高流动性
- ✅ 支持多种交易对

### 交易信息
- **基础货币**: USDC
- **杠杆**: 最高 50x
- **交易模式**: 全仓模式
- **结算**: 实时结算

## 📚 API 参考

### 端点
- **主网**: https://api.hyperliquid.xyz
- **测试网**: https://api.hyperliquid-testnet.xyz

### 主要接口
- `POST /info` - 查询信息（账户、持仓、价格等）
- `POST /exchange` - 执行交易（需要签名）

### 数据格式
```json
{
  "type": "clearinghouseState",
  "user": "0x你的地址"
}
```

## 🔮 未来计划

### 可能的增强功能
- [ ] 支持硬件钱包集成（Ledger, Trezor）
- [ ] 支持 WalletConnect 集成
- [ ] 实现只读模式的详细分析
- [ ] 添加历史交易查询

### 社区贡献

如果你有兴趣为 Hyperliquid 交易功能添加安全的签名支持，欢迎贡献！

建议方案：
1. 集成硬件钱包
2. 使用加密的密钥库（类似 eth-keystore）
3. 实现交易确认机制

## 🔗 相关链接

- **官网**: https://hyperliquid.xyz
- **文档**: https://hyperliquid.gitbook.io
- **Discord**: https://discord.gg/hyperliquid
- **Twitter**: https://twitter.com/HyperliquidX

---

**💡 提示**: 如果你主要交易在 Hyperliquid 上，建议使用本系统的查询功能监控账户状态，同时在其他中心化交易所执行自动跟单策略。
