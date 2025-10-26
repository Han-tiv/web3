# 🪙 多币种交易指南 - DeepSeek AI Trading Bot

**支持币种**: BTC, ETH, XRP, SOL, BNB, DOGE

---

## 💎 支持的币种

| 币种 | 全名 | 交易对 | 最小交易量 | 参考价格 |
|------|------|--------|-----------|---------|
| **BTC** | Bitcoin | BTC/USDT | 0.0001 | $113,626 |
| **ETH** | Ethereum | ETH/USDT | 0.001 | $4,068 |
| **XRP** | Ripple | XRP/USDT | 1.0 | $2.50 |
| **SOL** | Solana | SOL/USDT | 0.01 | $199 |
| **BNB** | Binance Coin | BNB/USDT | 0.01 | $1,129 |
| **DOGE** | Dogecoin | DOGE/USDT | 1.0 | $0.2024 |

---

## 🚀 快速开始

### 方式 1: 使用环境变量（推荐）

编辑 `.env` 文件：

```bash
cd /home/hanins/code/web3
nano .env
```

添加币种配置：

```bash
# 选择交易币种 (BTC/ETH/XRP/SOL/BNB/DOGE)
TRADING_SYMBOL=BTC

# API 密钥
DEEPSEEK_API_KEY=sk-your-key
GATE_API_KEY=your-key
GATE_SECRET=your-secret
```

### 方式 2: 启动时指定

```bash
# 交易比特币
TRADING_SYMBOL=BTC ./scripts/run_deepseek_gate.sh

# 交易以太坊
TRADING_SYMBOL=ETH ./scripts/run_deepseek_gate.sh

# 交易 Ripple
TRADING_SYMBOL=XRP ./scripts/run_deepseek_gate.sh

# 交易 Solana
TRADING_SYMBOL=SOL ./scripts/run_deepseek_gate.sh

# 交易 BNB
TRADING_SYMBOL=BNB ./scripts/run_deepseek_gate.sh

# 交易狗狗币
TRADING_SYMBOL=DOGE ./scripts/run_deepseek_gate.sh
```

---

## 💰 不同币种的资金配置

### 100 USDT 账户建议配置

**BTC 交易** (高价值):
```
基础投入: 100 USDT
最小交易: 0.0001 BTC (~$11)
推荐仓位: 0.0001-0.002 BTC
单笔占用: ~$11-$220
适合账户: 100 USDT+
```

**ETH 交易** (中高价值):
```
基础投入: 100 USDT
最小交易: 0.001 ETH (~$4)
推荐仓位: 0.001-0.03 ETH
单笔占用: ~$4-$120
适合账户: 50 USDT+
```

**XRP 交易** (低价值):
```
基础投入: 100 USDT
最小交易: 1 XRP (~$2.50)
推荐仓位: 1-40 XRP
单笔占用: ~$2.50-$100
适合账户: 30 USDT+
```

**SOL 交易** (中等价值):
```
基础投入: 100 USDT
最小交易: 0.01 SOL (~$2)
推荐仓位: 0.01-0.5 SOL
单笔占用: ~$2-$100
适合账户: 30 USDT+
```

**BNB 交易** (中高价值):
```
基础投入: 100 USDT
最小交易: 0.01 BNB (~$11)
推荐仓位: 0.01-0.1 BNB
单笔占用: ~$11-$110
适合账户: 50 USDT+
```

**DOGE 交易** (低价值):
```
基础投入: 100 USDT
最小交易: 1 DOGE (~$0.2)
推荐仓位: 1-500 DOGE
单笔占用: ~$0.2-$100
适合账户: 20 USDT+
```

---

## 📊 智能仓位计算示例

### BTC 交易 (价格 $67,000)

**场景 1**: HIGH 信心 + 强势上涨
```
基础USDT: 100
信心倍数: 1.5x
趋势倍数: 1.2x
RSI倍数: 1.0x
───────────────
最终USDT: 180
BTC数量: 0.00268
```

**场景 2**: MEDIUM 信心 + 震荡
```
基础USDT: 100
信心倍数: 1.0x
趋势倍数: 1.0x
RSI倍数: 1.0x
───────────────
最终USDT: 100
BTC数量: 0.00149
```

### ETH 交易 (价格 $4,000)

**场景 1**: HIGH 信心 + 强势上涨
```
基础USDT: 100
最终USDT: 180
ETH数量: 0.045
```

**场景 2**: MEDIUM 信心
```
基础USDT: 100
最终USDT: 100
ETH数量: 0.025
```

### XRP 交易 (价格 $2.50)

**场景 1**: HIGH 信心 + 强势上涨
```
基础USDT: 100
最终USDT: 180
XRP数量: 72
```

**场景 2**: MEDIUM 信心
```
基础USDT: 100
最终USDT: 100
XRP数量: 40
```

### DOGE 交易 (价格 $0.20)

**场景 1**: HIGH 信心 + 强势上涨
```
基础USDT: 100
最终USDT: 180
DOGE数量: 900
```

**场景 2**: LOW 信心
```
基础USDT: 100
最终USDT: 50
DOGE数量: 250
```

---

## 🎯 使用场景推荐

### 大账户 (500+ USDT)

**建议**: 主要交易 BTC
- 流动性好
- 波动相对稳定
- AI 分析数据充足
- 适合大资金

**配置**:
```bash
TRADING_SYMBOL=BTC
```

### 中等账户 (100-500 USDT)

**建议**: BTC 或 ETH
- BTC: 稳健型
- ETH: 进取型
- 两者流动性都很好

**配置**:
```bash
# 稳健
TRADING_SYMBOL=BTC

# 进取
TRADING_SYMBOL=ETH
```

### 小账户 (50-100 USDT)

**建议**: ETH、SOL 或 DOGE
- ETH: 流动性好，价格适中
- SOL: 波动大，机会多
- DOGE: 价格低，灵活性高

**配置**:
```bash
TRADING_SYMBOL=ETH  # 推荐
# 或
TRADING_SYMBOL=SOL  # 激进
# 或
TRADING_SYMBOL=DOGE # 超小资金
```

### 测试账户 (< 50 USDT)

**建议**: SOL 或 DOGE
- 最小投入要求低
- 适合测试策略

**配置**:
```bash
TRADING_SYMBOL=DOGE
```

---

## 📈 不同币种的特点

### BTC (Bitcoin)
- ✅ 最成熟的加密货币
- ✅ 流动性最好
- ✅ 数据最完整
- ✅ AI 分析最准确
- ⚠️  价格高，小资金不友好
- ⚠️  波动相对较小

### ETH (Ethereum)
- ✅ 流动性好
- ✅ 价格适中
- ✅ 技术指标有效
- ✅ 适合中等资金
- ⚠️  受 DeFi 影响大
- ⚠️  偶尔剧烈波动

### XRP (Ripple)
- ✅ 价格低，适合小资金
- ✅ 支付网络成熟
- ✅ 流动性好
- ✅ 波动相对稳定
- ⚠️  受监管政策影响大
- ⚠️  中心化程度较高

### SOL (Solana)
- ✅ 价格适中
- ✅ 波动大，机会多
- ✅ 适合小资金
- ⚠️  流动性略差
- ⚠️  容易出现极端行情
- ⚠️  AI 预测难度较高

### BNB (Binance Coin)
- ✅ Binance 生态支撑
- ✅ 相对稳定
- ✅ 流动性好
- ⚠️  价格中高
- ⚠️  受 Binance 新闻影响

### DOGE (Dogecoin)
- ✅ 价格极低，灵活
- ✅ 社区活跃
- ✅ 适合超小资金
- ⚠️  极度投机
- ⚠️  受社交媒体影响大
- ⚠️  AI 预测准确性低

---

## 🔄 币种切换

### 停止当前交易

1. 按 `Ctrl + C` 停止程序
2. 等待当前周期完成（如果正在执行）
3. 检查是否有未平仓位

### 切换币种

```bash
# 方式 1: 修改 .env
nano /home/hanins/code/web3/.env
# 修改 TRADING_SYMBOL=新币种

# 方式 2: 直接指定
TRADING_SYMBOL=ETH ./scripts/run_deepseek_gate.sh
```

### 注意事项

⚠️  **切换币种前必须平掉所有持仓！**

程序不会自动平仓旧币种，你需要：
1. 停止程序
2. 手动到 Gate.io 平掉持仓
3. 或者等待 AI 自动平仓
4. 确认无持仓后再切换

---

## 💡 最佳实践

### 1. 币种选择策略

**保守型** (推荐新手):
```
BTC > ETH > BNB
稳定性高，流动性好
```

**平衡型** (有经验):
```
BTC + ETH 轮换
根据市场情况切换
```

**激进型** (高风险):
```
SOL + DOGE
高波动，高收益/高风险
```

### 2. 多币种组合

**不推荐**: 同时运行多个币种
- 资金分散
- 风险增加
- 管理复杂

**推荐**: 单一币种专注
- 每次只交易一个币种
- 根据市场情况切换
- 周为单位调整策略

### 3. 币种评估周期

建议每周评估一次：

```
周一: 查看上周表现
周二: 决定是否切换币种
周三-周日: 执行交易
```

评估指标：
- AI 信号准确率
- 实际盈亏
- 市场趋势
- 流动性状况

---

## 🎯 启动示例

### BTC 交易 (默认)

```bash
cd /home/hanins/code/web3/apps/rust-trading-bot
cargo build --release --bin deepseek_trader
./scripts/run_deepseek_gate.sh
```

输出：
```
═══════════════════════════════════════════
🤖 DeepSeek AI Trading Bot v3.0 - Multi-Coin
═══════════════════════════════════════════

💎 支持的交易币种:
   - Bitcoin (BTC/USDT)
   - Ethereum (ETH/USDT)
   - Solana (SOL/USDT)
   - Binance Coin (BNB/USDT)
   - Dogecoin (DOGE/USDT)

✅ 当前选择: Bitcoin (BTC/USDT)

📊 交易配置:
   币种: Bitcoin
   交易对: BTC/USDT
   K线周期: 15m
   最小交易量: 0.0001 BTC
   杠杆倍数: 5x
   执行间隔: 15 分钟
   交易所: Gate
```

### ETH 交易

```bash
TRADING_SYMBOL=ETH ./scripts/run_deepseek_gate.sh
```

输出：
```
✅ 当前选择: Ethereum (ETH/USDT)

📊 交易配置:
   币种: Ethereum
   交易对: ETH/USDT
   最小交易量: 0.001 ETH
   ...
```

### DOGE 交易

```bash
TRADING_SYMBOL=DOGE ./scripts/run_deepseek_gate.sh
```

输出：
```
✅ 当前选择: Dogecoin (DOGE/USDT)

📊 交易配置:
   币种: Dogecoin
   交易对: DOGE/USDT
   最小交易量: 1.0 DOGE
   ...
```

---

## ⚠️  风险提示

### 不同币种的风险级别

| 币种 | 风险等级 | 适合人群 |
|------|---------|---------|
| BTC | 🟢 低 | 所有人 |
| ETH | 🟡 中 | 有经验者 |
| BNB | 🟡 中 | 有经验者 |
| SOL | 🟠 高 | 激进投资者 |
| DOGE | 🔴 极高 | 高风险偏好 |

### 各币种特殊风险

**BTC**:
- 价格高，小资金不灵活
- 需要较大账户才能有效交易

**ETH**:
- 受以太坊网络升级影响
- DeFi 事件可能导致剧烈波动

**SOL**:
- 网络宕机风险
- 价格波动极大
- 流动性可能不足

**BNB**:
- 高度依赖 Binance
- 监管风险
- Binance 新闻敏感

**DOGE**:
- 纯投机币种
- 社交媒体操纵风险
- 基本面分析无效
- 极端波动

### 通用建议

1. **小额测试**: 先用小资金测试各币种
2. **风险分散**: 不要把所有资金投入一个币种
3. **及时止损**: 设置账户总止损 (如 20%)
4. **保持警惕**: 加密货币市场 7×24 运行
5. **定期评估**: 每周检查策略有效性

---

## 📞 快速命令参考

```bash
# 查看支持的币种
cat /home/hanins/code/web3/MULTI_COIN_TRADING_GUIDE.md

# 编译程序
cd /home/hanins/code/web3/apps/rust-trading-bot
cargo build --release --bin deepseek_trader

# 交易不同币种
TRADING_SYMBOL=BTC ./scripts/run_deepseek_gate.sh  # 比特币
TRADING_SYMBOL=ETH ./scripts/run_deepseek_gate.sh  # 以太坊
TRADING_SYMBOL=XRP ./scripts/run_deepseek_gate.sh  # Ripple
TRADING_SYMBOL=SOL ./scripts/run_deepseek_gate.sh  # Solana
TRADING_SYMBOL=BNB ./scripts/run_deepseek_gate.sh  # BNB
TRADING_SYMBOL=DOGE ./scripts/run_deepseek_gate.sh # 狗狗币

# 停止交易
Ctrl + C

# 检查编译
cargo check --bin deepseek_trader
```

---

## 🎊 总结

### 推荐配置

**新手 (100 USDT)**:
```bash
TRADING_SYMBOL=BTC  # 最稳健
```

**有经验 (200+ USDT)**:
```bash
TRADING_SYMBOL=ETH  # 平衡收益风险
```

**激进 (50 USDT)**:
```bash
TRADING_SYMBOL=DOGE # 高波动高风险
```

---

**🚀 选择你的币种，开始交易！**

```bash
cd /home/hanins/code/web3/apps/rust-trading-bot
TRADING_SYMBOL=你选择的币种 ./scripts/run_deepseek_gate.sh
```

**⚠️ 风险自负，谨慎交易！**

---

_文档更新: 2025-10-26_  
_版本: v3.0 Multi-Coin_
