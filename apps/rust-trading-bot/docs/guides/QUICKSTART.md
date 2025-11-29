# 🚀 快速开始 - ds.py 功能整合版

## 1️⃣ 配置环境变量

```bash
# 复制配置模板
cp .env.example .env

# 编辑 .env 文件
nano .env
```

**必需配置**:
```bash
EXCHANGE_TYPE=GATE                          # 交易所类型
API_KEY=your_gate_api_key                   # 交易所 API Key
API_SECRET=your_gate_api_secret             # 交易所 API Secret
DEEPSEEK_API_KEY=sk-your_deepseek_key       # DeepSeek API Key
TRADING_SYMBOL=BTC                          # 交易币种
```

**可选但推荐**:
```bash
# CryptoOracle 市场情绪数据（强烈推荐！）
CRYPTO_ORACLE_API_KEY=7ad48a56-8730-4238-a714-eebc30834e3e
```

## 2️⃣ 编译运行

```bash
# 编译
cargo build --release --bin deepseek_trader

# 运行
cargo run --release --bin deepseek_trader
```

## 3️⃣ 验证整合功能

运行后查看日志，应该看到：

```
✅ 检测到 CRYPTO_ORACLE_API_KEY，将使用 CryptoOracle 情绪数据
📊 CryptoOracle: 【市场情绪】乐观52.3% 悲观47.7% 净值+0.046
📊 市场乐观情绪较强
```

如果没有配置 CryptoOracle，会自动降级：
```
⚠️  获取 CryptoOracle 情绪失败: ...
📊 Fear & Greed: 65 (Greed)
```

## 4️⃣ 新增功能概览

### ✅ CryptoOracle 情绪数据
- 实时市场情绪（15分钟更新）
- 乐观/悲观比例分析
- 自动检测数据延迟

### ✅ 增强 AI Prompt
- 防频繁交易原则（4条）
- 交易权重分配（60%技术/30%情绪/10%风控）
- 智能仓位管理规则（4条）
- 趋势自动分析

### ✅ 更稳定的信号
- 减少过度保守的 HOLD
- 更好的趋势跟随能力
- BTC 多头偏好

## 5️⃣ 查看详细文档

```bash
# 完整整合说明
cat INTEGRATION_NOTES.md

# 环境变量配置
cat .env.example
```

---

**快速对比**：

| 功能 | 整合前 | 整合后 |
|------|--------|--------|
| 情绪数据源 | 1个 | ✅ 2个（CryptoOracle优先） |
| AI Prompt | 50行 | ✅ 150行详细规则 |
| 防频繁交易 | ❌ | ✅ 4条原则 |
| 智能仓位 | 基础 | ✅ 4条规则 |

**Happy Trading! 🚀**
