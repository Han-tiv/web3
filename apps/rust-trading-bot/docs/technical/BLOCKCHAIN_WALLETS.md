# 🔗 区块链钱包集成

## 📋 概述

系统现已集成 **BSC (Binance Smart Chain)** 和 **Solana** 链上钱包，支持查询各类代币余额。

---

## 🌐 支持的区块链

| 区块链 | 状态 | 原生代币 | 支持代币 |
|--------|------|----------|----------|
| **BSC** | ✅ | BNB | USDT (BEP20), BUSD |
| **Solana** | ✅ | SOL | - |

---

## 🔧 配置说明

### BSC 钱包配置

在 `.env` 文件中添加：

```bash
# BSC (Binance Smart Chain) 钱包
BSC_ADDRESS=0x你的BSC地址
BSC_PRIVATE_KEY=0x你的私钥
BSC_TESTNET=false  # false=主网, true=测试网
```

**支持查询**：
- BNB（原生代币）
- USDT (BEP20): `0x55d398326f99059fF775485246999027B3197955`
- BUSD: `0xe9e7CEA3DedcA5984780Bafc599bD69ADd087D56`

### Solana 钱包配置

```bash
# Solana 钱包
SOLANA_ADDRESS=你的Solana公钥地址
SOLANA_PRIVATE_KEY=你的私钥（base58编码）
SOLANA_TESTNET=false  # false=主网, true=开发网
```

**支持查询**：
- SOL（原生代币）

**注意**：SPL Token（USDC, USDT等）查询因依赖冲突暂未实现，可通过 Solana 浏览器查看。

---

## 📊 功能支持

### BSC 功能

| 功能 | 状态 | 说明 |
|------|------|------|
| 查询 BNB 余额 | ✅ | 原生代币 |
| 查询 USDT 余额 | ✅ | BEP20 标准 |
| 查询 BUSD 余额 | ✅ | BEP20 标准 |
| 合约交易 | ❌ | 钱包仅查询，交易请使用 DEX |

### Solana 功能

| 功能 | 状态 | 说明 |
|------|------|------|
| 查询 SOL 余额 | ✅ | 原生代币 |
| 查询 SPL Token | ❌ | 依赖冲突，暂不支持 |
| 合约交易 | ❌ | 钱包仅查询，交易请使用 DEX |

---

## 💡 使用示例

### 查看所有资产

```bash
cargo run --release --bin show_assets
```

输出示例：
```
┌──────────────────────────────────────────┐
│ 🏦 BSC                                   │
├──────────────────────────────────────────┤
│ 💰 账户余额
│   总余额:              150.50 USDT
│   可用余额:            150.50 USDT
│
│ 详细：
│   BNB: 0.5 (150.00 USD)
│   USDT: 100.00
│   BUSD: 50.50
└──────────────────────────────────────────┘

┌──────────────────────────────────────────┐
│ 🏦 Solana                                │
├──────────────────────────────────────────┤
│ 💰 账户余额
│   总余额:               75.00 USDT
│   可用余额:             75.00 USDT
│
│ 详细：
│   SOL: 0.5 (75.00 USD)
└──────────────────────────────────────────┘
```

---

## 🏗️ 技术实现

### BSC 实现

- 使用 **ethers-rs** 库
- 通过 RPC 节点查询链上数据
- ERC20 `balanceOf` 方法查询代币余额
- 支持主网和测试网切换

**RPC 节点**：
- 主网：`https://bsc-dataseed1.binance.org`
- 测试网：`https://data-seed-prebsc-1-s1.binance.org:8545`

### Solana 实现

- 使用 **HTTP RPC** 调用（避免依赖冲突）
- JSON-RPC `getBalance` 方法
- 支持主网和开发网切换

**RPC 节点**：
- 主网：`https://api.mainnet-beta.solana.com`
- 开发网：`https://api.devnet.solana.com`

---

## ⚠️ 安全提示

### 私钥安全

1. **专用钱包**：建议使用专门的钱包地址，不要使用主钱包
2. **文件权限**：设置 `.env` 文件权限为 600
   ```bash
   chmod 600 .env
   ```
3. **不要提交**：确保 `.env` 在 `.gitignore` 中
4. **定期更换**：定期更换私钥

### RPC 节点

- 公共 RPC 节点可能有速率限制
- 生产环境建议使用付费节点（如 Infura, QuickNode）
- 可配置自定义 RPC 节点

---

## 📈 价格估算

系统使用固定汇率估算代币价值（单位：USDT）：

| 代币 | 估算价格 |
|------|---------|
| BNB  | $300 |
| SOL  | $150 |
| USDT | $1 |
| BUSD | $1 |

**注意**：这些是估算价格，实际价值以市场价格为准。

---

## 🔮 未来计划

### 短期
- [ ] 添加更多 ERC20/BEP20 代币支持
- [ ] 实现 SPL Token 查询（解决依赖冲突）
- [ ] 添加实时价格查询（通过 DEX 或预言机）

### 中期
- [ ] 支持以太坊主网
- [ ] 支持 Polygon 网络
- [ ] 添加 NFT 余额查询

### 长期
- [ ] DEX 交易集成（PancakeSwap, Jupiter）
- [ ] 跨链桥接支持
- [ ] DeFi 协议集成

---

## 🎯 系统概览

### 完整支持列表

**中心化交易所**（CEX）- 6 个：
1. Binance - 完整交易 ✅
2. OKX - 完整交易 ✅
3. Bitget - 完整交易 ✅
4. Bybit - 完整交易 ✅
5. Gate.io - 完整交易 ✅
6. Hyperliquid - 完整交易 ✅（DEX）

**区块链钱包** - 2 个：
1. BSC - 余额查询 ✅
2. Solana - 余额查询 ✅

**总计**：**8 个平台**全面监控！

---

## 📚 相关文档

- [多交易所使用指南](./README_MULTI_EXCHANGE.md)
- [Hyperliquid 使用说明](./HYPERLIQUID_README.md)
- [系统总览](./README.md)

---

## 💬 常见问题

### Q: 为什么 Solana 不支持 SPL Token？
A: Solana SDK 与 ethers 库存在依赖冲突。当前使用 HTTP RPC 仅查询 SOL 原生代币。未来会通过其他方式解决。

### Q: BSC 可以交易吗？
A: 当前仅支持余额查询。如需交易，请使用 DEX（如 PancakeSwap）或 CEX。

### Q: 如何添加其他代币？
A: 修改 `src/bsc_wallet.rs` 或 `src/solana_wallet.rs`，添加对应的合约地址和查询逻辑。

### Q: 为什么使用固定价格估算？
A: 为简化实现。可通过集成 Chainlink 或 DEX 价格预言机获取实时价格。

---

**🎊 区块链钱包集成完成！现在可以跨平台监控所有资产了！** 🚀
