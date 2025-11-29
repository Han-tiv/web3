# ⚠️  环境变量配置指南

## 问题说明

重构过程中，`.env` 文件被重写为只包含 `USE_VALUESCAN_V2=true`，导致程序启动失败。

**错误信息**: `environment variable not found`

---

## 📋 必需的环境变量

程序需要以下环境变量才能启动：

| 变量名 | 说明 | 必需 |
|--------|------|------|
| `DEEPSEEK_API_KEY` | DeepSeek API密钥 | ✅ 必需 |
| `GEMINI_API_KEY` | Gemini API密钥 | ✅ 必需 |
| `BINANCE_API_KEY` | 币安API密钥 | ✅ 必需 |
| `BINANCE_SECRET` | 币安Secret密钥 | ✅ 必需 |
| `BINANCE_TESTNET` | 是否使用测试网 | ⚙️  可选 (默认false) |
| `USE_VALUESCAN_V2` | 使用DeepSeek开仓 | ⚙️  可选 (已配置true) |

---

## 🔍 查找旧配置

### 方法1: 检查系统环境变量
```bash
# 查看当前环境变量
env | grep -E "DEEPSEEK|GEMINI|BINANCE"

# 检查shell配置文件
grep -E "DEEPSEEK|GEMINI|BINANCE" ~/.bashrc ~/.zshrc 2>/dev/null
```

### 方法2: 检查旧程序进程

如果旧程序还在运行（PID: 3203533），可以查看其环境变量：
```bash
# 查看进程环境变量
cat /proc/3203533/environ | tr '\0' '\n' | grep -E "DEEPSEEK|GEMINI|BINANCE"
```

### 方法3: 检查备份文件
```bash
# 查找可能的备份
find ~ -name ".env.backup*" -o -name ".env.old" 2>/dev/null

# 查找其他.env文件
find /home/hanins/code/web3 -name ".env" -type f 2>/dev/null
```

---

## ✅ 配置.env文件

### 完整示例

创建 `/home/hanins/code/web3/apps/rust-trading-bot/.env`:

```bash
# ═══════════════════════════════════════════════════════════
# AI API配置
# ═══════════════════════════════════════════════════════════

# DeepSeek API密钥 (必需)
DEEPSEEK_API_KEY=sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# Gemini API密钥 (必需)
GEMINI_API_KEY=AIzaxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# ═══════════════════════════════════════════════════════════
# 币安API配置
# ═══════════════════════════════════════════════════════════

# 币安API密钥 (必需)
BINANCE_API_KEY=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# 币安Secret密钥 (必需)
BINANCE_SECRET=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# 是否使用测试网 (可选，默认false)
BINANCE_TESTNET=false

# ═══════════════════════════════════════════════════════════
# AI分工配置
# ═══════════════════════════════════════════════════════════

# 使用Valuescan V2版本 (DeepSeek开仓分析)
# true  = DeepSeek V2 开仓分析 + Gemini 持仓管理  ← 当前配置
# false = Gemini V1 开仓分析 + Gemini 持仓管理
USE_VALUESCAN_V2=true

# ═══════════════════════════════════════════════════════════
# 其他可选配置
# ═══════════════════════════════════════════════════════════

# 日志级别 (可选)
# RUST_LOG=info

# Web服务器端口 (可选，默认8080)
# WEB_SERVER_PORT=8080

# 数据库路径 (可选，默认data/trading.db)
# DATABASE_PATH=data/trading.db
```

---

## 🚀 启动步骤

### 1. 补全环境变量

```bash
# 编辑.env文件
vim .env

# 或使用nano
nano .env
```

### 2. 验证配置

```bash
# 检查.env文件内容
cat .env

# 确保所有必需变量都已设置
grep -E "DEEPSEEK_API_KEY|GEMINI_API_KEY|BINANCE_API_KEY|BINANCE_SECRET" .env
```

### 3. 启动程序

```bash
# 后台启动
nohup ./target/release/integrated_ai_trader > logs/output.log 2>&1 &

# 查看日志
tail -f logs/startup.log
```

### 4. 验证启动

```bash
# 检查进程
ps aux | grep integrated_ai_trader

# 检查Web服务器
curl http://localhost:8080/health

# 查看日志中的AI配置
tail -100 logs/startup.log | grep -E "Valuescan|AI|DeepSeek|Gemini"
```

---

## 📊 当前状态

### ✅ 已完成

- ✅ 数据库已迁移 (删除recommend_action字段)
- ✅ 代码已修改 (删除信号过滤逻辑)
- ✅ 已编译完成 (Release模式)
- ✅ AI分工已配置 (USE_VALUESCAN_V2=true)
- ✅ 数据库已备份 (`data/trading.db.backup_20251129_213610`)

### ❌ 待完成

- ❌ 补全环境变量配置
- ❌ 启动程序

---

## 🔄 回滚方案

如果需要回滚数据库（不建议，新表结构更好）：

```bash
# 恢复数据库备份
cp data/trading.db.backup_20251129_213610 data/trading.db

# 但代码已经修改，需要一起回滚
# 建议: 保留新代码和新数据库，只需配置环境变量即可
```

---

## 💡 提示

1. **API密钥安全**: 确保`.env`文件在`.gitignore`中，避免泄露
2. **测试网**: 首次启动建议使用测试网 (`BINANCE_TESTNET=true`)
3. **日志监控**: 启动后密切关注日志，确认AI配置正确
4. **新架构优势**: 重构后系统更简洁，所有信号都会被AI智能分析

---

## 🎯 期望结果

配置完成后，启动日志应该显示：

```
[xx:xx:xx] 🚀 集成AI交易系统启动
[xx:xx:xx] 📦 版本: 2.0.0-refactored
[xx:xx:xx] 🤖 Valuescan版本: V2 (USE_VALUESCAN_V2=true)
[xx:xx:xx] ✅ DeepSeek客户端初始化成功
[xx:xx:xx] ✅ Gemini客户端初始化成功
[xx:xx:xx] ✅ 所有系统组件已启动完成
```

等待新信号到来后：

```
[xx:xx:xx] 📡 轮询到 X 条待处理的Telegram信号
[xx:xx:xx]   📨 处理信号: BTCUSDT
[xx:xx:xx] 🧠 开始AI分析: BTCUSDT
[xx:xx:xx] 🤖 Valuescan版本: V2
[xx:xx:xx] 🎯 AI决策: ENTER | 信心: 8
```

**不再有 "⏭️ 跳过非BUY信号"** ✅

---

<div align="center">

**补全环境变量后，重构即可完成！** 🚀

</div>
