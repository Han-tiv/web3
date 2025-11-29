# 🚀 混合架构交易系统部署指南

## 📋 架构概览

```
┌─────────────────────────────────────────────────────────────────┐
│                         用户层                                   │
│   Web控制台 (http://localhost:5173)                             │
└────────────────────────┬────────────────────────────────────────┘
                         │
┌────────────────────────┴────────────────────────────────────────┐
│                      应用层                                      │
│  ┌──────────────────┐         ┌──────────────────────┐        │
│  │ Python监控模块    │  HTTP   │ Rust交易引擎          │        │
│  │ (Telethon)       │ ──────> │ (Binance API)        │        │
│  │ :telegram        │ POST    │ :trading :AI :risk   │        │
│  │                  │ /signals│                      │        │
│  └──────────────────┘         └──────────────────────┘        │
└─────────────────────────────────────────────────────────────────┘
                         │                    │
┌────────────────────────┴────────────────────┴───────────────────┐
│                      数据层                                      │
│  SQLite数据库 (data/trading.db)                                 │
│  - telegram_signals (信号记录)                                  │
│  - ai_analysis (AI分析记录)                                     │
│  - trades (交易历史)                                            │
└─────────────────────────────────────────────────────────────────┘
```

## ✅ 优势说明

### 为什么采用混合架构?

**问题背景**:
- 原Rust monolith使用grammers库连接Telegram
- 遇到频繁断线(498错误,8小时内数百次)
- Rust生态Telegram库不够成熟

**解决方案**:
- **Python (Telethon)**: 专门负责Telegram监控
  - ✅ 稳定性: Telethon有9k+ stars,数百万下载
  - ✅ 易维护: Python代码简洁,调试方便
  - ✅ 社区支持: 丰富的文档和问题解决方案

- **Rust (Trading Engine)**: 专门负责交易执行
  - ✅ 高性能: 订单执行、持仓管理
  - ✅ 类型安全: 编译时错误检查
  - ✅ 内存安全: 无GC,低延迟

- **HTTP通信**: 简单可靠的进程间通信
  - ✅ 解耦: 两个模块独立部署和升级
  - ✅ 可观测: HTTP请求可以轻松监控和调试
  - ✅ 标准化: JSON格式,易于扩展

---

## 🔧 系统要求

### 软件依赖

```bash
# Python 3.8+
python3 --version

# Rust 1.70+
rustc --version

# Node.js 18+ (前端)
node --version
```

### 必需的配置文件

确保 `/home/hanins/code/web3/.env` 包含以下配置:

```bash
# Telegram配置
TELEGRAM_API_ID=2040
TELEGRAM_API_HASH=b18441a1ff607e10a989891a5462e627
TELEGRAM_PHONE=+17578852234

# 监控的频道列表 (逗号分隔)
TELEGRAM_CHANNELS=-1001234567890,@trading_signals

# Binance配置
BINANCE_API_KEY=your_api_key
BINANCE_SECRET=your_secret
BINANCE_TESTNET=false

# AI服务配置
DEEPSEEK_API_KEY=sk-xxx
GEMINI_API_KEY=xxx

# Rust引擎地址
RUST_ENGINE_URL=http://localhost:8080
RUST_ENGINE_TIMEOUT=5

# 日志配置
LOG_LEVEL=INFO
LOG_FILE=telegram_monitor.log
```

---

## 📦 安装步骤

### 第1步: 安装Python依赖

```bash
cd /home/hanins/code/web3/apps/python-telegram-monitor
pip3 install -r requirements.txt
```

### 第2步: 编译Rust引擎

```bash
cd /home/hanins/code/web3/apps/rust-trading-bot
cargo build --release
```

### 第3步: 首次Telegram登录

```bash
cd /home/hanins/code/web3/apps/python-telegram-monitor
python3 telegram_monitor.py
```

首次运行时会要求输入Telegram验证码,完成后会生成 `telegram_session.session` 文件。

---

## 🚀 启动系统

### 方式1: 分别启动 (推荐用于测试)

#### 终端1: 启动Rust交易引擎

```bash
cd /home/hanins/code/web3/apps/rust-trading-bot
bash start_trader.sh
```

检查状态:
```bash
curl http://localhost:8080/health
# 应返回: OK
```

#### 终端2: 启动Python监控

```bash
cd /home/hanins/code/web3/apps/python-telegram-monitor
bash start_monitor.sh
```

检查日志:
```bash
tail -f telegram_monitor.log
```

#### 终端3: 启动Web前端

```bash
cd /home/hanins/code/web3/apps/rust-trading-bot/web
npm run dev
```

访问: http://localhost:5173

---

### 方式2: 后台运行 (生产环境)

#### 1. 启动Rust引擎

```bash
cd /home/hanins/code/web3/apps/rust-trading-bot
nohup cargo run --release --bin integrated_ai_trader > trader.log 2>&1 &
echo $! > trader.pid
```

#### 2. 启动Python监控

```bash
cd /home/hanins/code/web3/apps/python-telegram-monitor
nohup python3 telegram_monitor.py > monitor.log 2>&1 &
echo $! > monitor.pid
```

#### 3. 检查进程

```bash
ps aux | grep integrated_ai_trader
ps aux | grep telegram_monitor
```

---

## 🧪 测试集成

运行集成测试脚本:

```bash
cd /home/hanins/code/web3/apps/python-telegram-monitor
bash test_integration.sh
```

测试内容:
1. ✅ Rust引擎健康检查
2. ✅ 发送测试信号
3. ✅ 验证数据库保存
4. ✅ 多信号并发测试

---

## 📊 监控和调试

### 查看Rust引擎日志

```bash
cd /home/hanins/code/web3/apps/rust-trading-bot
tail -f trader.log

# 或实时监控
tail -f trader.log | grep -E "📨|✅|❌|🚨"
```

### 查看Python监控日志

```bash
cd /home/hanins/code/web3/apps/python-telegram-monitor
tail -f telegram_monitor.log

# 或实时监控
tail -f telegram_monitor.log | grep -E "📨|✅|❌|⚠️"
```

### 系统状态查询

```bash
# 检查Rust引擎状态
curl -s http://localhost:8080/api/status | jq '.'

# 查看最近的Telegram信号
curl -s http://localhost:8080/api/telegram-signals | jq '.'

# 查看AI分析记录
curl -s http://localhost:8080/api/ai-history | jq '.'

# 查看当前持仓
curl -s http://localhost:8080/api/positions | jq '.'
```

---

## 🔍 故障排查

### 问题1: Rust引擎无法启动

**症状**: `curl http://localhost:8080/health` 超时

**排查**:
```bash
# 检查端口占用
netstat -tlnp | grep 8080

# 检查Binance API权限
curl http://localhost:8080/api/status
```

**解决**:
- 确保Binance API权限开启 (Enable Reading + Enable Futures)
- 检查 `.env` 文件配置是否正确

---

### 问题2: Python监控连接失败

**症状**: `telegram_monitor.log` 显示连接错误

**排查**:
```bash
# 检查Telegram配置
python3 -c "from config import validate_config; validate_config()"

# 测试网络连接
ping telegram.org
```

**解决**:
- 删除 `telegram_session.session` 重新登录
- 检查网络代理设置
- 确认 `TELEGRAM_API_ID` 和 `TELEGRAM_API_HASH` 正确

---

### 问题3: 信号未被Rust接收

**症状**: Python日志显示发送成功,但Rust没反应

**排查**:
```bash
# 测试HTTP连接
curl -X POST http://localhost:8080/api/signals \
    -H "Content-Type: application/json" \
    -d '{"symbol":"BTCUSDT","raw_message":"test","timestamp":1700000000}'

# 检查数据库
curl http://localhost:8080/api/telegram-signals | jq '.'
```

**解决**:
- 确认 `RUST_ENGINE_URL` 配置正确
- 检查防火墙设置
- 查看Rust引擎错误日志

---

## 🛑 停止系统

### 停止所有服务

```bash
# 停止Rust引擎
kill $(cat /home/hanins/code/web3/apps/rust-trading-bot/trader.pid)

# 停止Python监控
kill $(cat /home/hanins/code/web3/apps/python-telegram-monitor/monitor.pid)

# 或使用进程名
pkill -f integrated_ai_trader
pkill -f telegram_monitor
```

---

## 📈 性能指标

### Python监控模块
- 消息处理延迟: < 100ms
- 信号发送延迟: < 50ms (本地HTTP)
- 内存占用: ~50MB
- CPU占用: < 1%

### Rust交易引擎
- 订单执行延迟: < 200ms
- 持仓监控周期: 10分钟
- 内存占用: ~100MB
- CPU占用: < 5%

---

## 🔄 升级和维护

### 升级Python监控

```bash
cd /home/hanins/code/web3/apps/python-telegram-monitor
git pull
pip3 install -r requirements.txt --upgrade
bash start_monitor.sh  # 重启
```

### 升级Rust引擎

```bash
cd /home/hanins/code/web3/apps/rust-trading-bot
git pull
cargo build --release
bash start_trader.sh  # 重启
```

### 清理日志

```bash
# 归档旧日志
cd /home/hanins/code/web3/apps/python-telegram-monitor
mv telegram_monitor.log telegram_monitor.$(date +%Y%m%d).log
gzip telegram_monitor.$(date +%Y%m%d).log

# Rust日志同理
cd /home/hanins/code/web3/apps/rust-trading-bot
mv trader.log trader.$(date +%Y%m%d).log
gzip trader.$(date +%Y%m%d).log
```

---

## 📞 技术支持

### 相关文档

- Python监控: `/home/hanins/code/web3/apps/python-telegram-monitor/README.md`
- Rust引擎: `/home/hanins/code/web3/apps/rust-trading-bot/README.md`
- 项目根配置: `/home/hanins/code/web3/.claude/CLAUDE.md`

### 常用命令速查

```bash
# 启动完整系统
cd /home/hanins/code/web3/apps/rust-trading-bot && bash start_trader.sh &
cd /home/hanins/code/web3/apps/python-telegram-monitor && bash start_monitor.sh &

# 查看实时日志
tail -f apps/rust-trading-bot/trader.log apps/python-telegram-monitor/telegram_monitor.log

# 测试系统状态
curl http://localhost:8080/health && echo "Rust引擎正常"

# 查看最近信号
curl -s http://localhost:8080/api/telegram-signals | jq '.[] | {symbol, side, score, timestamp}' | head -20
```

---

**最后更新**: 2025-11-09
**架构版本**: v2.0 (混合架构)
**维护者**: AI Trading Team
