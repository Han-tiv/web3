# 🚀 混合架构交易系统 - 快速启动指南

## ⚡ 一键启动 (最简单)

```bash
cd /home/hanins/code/web3
bash start_trading.sh
```

等待启动完成,看到 `✅ 系统启动完成!` 即可。

---

## 🛑 一键停止

```bash
cd /home/hanins/code/web3
bash stop_trading.sh
```

---

## 🔍 快速检查

### 1. 系统是否在线?
```bash
curl http://localhost:8080/health
# 应返回: OK
```

### 2. 查看系统状态
```bash
curl -s http://localhost:8080/api/status | jq '.'
```

### 3. 查看最近的Telegram信号
```bash
curl -s http://localhost:8080/api/telegram-signals | jq '.[] | {symbol, side, timestamp}' | head -20
```

### 4. 查看当前持仓
```bash
curl -s http://localhost:8080/api/positions | jq '.'
```

### 5. 查看交易历史
```bash
curl -s http://localhost:8080/api/trades?limit=10 | jq '.'
```

---

## 📊 实时监控

### 同时监控两个日志
```bash
tail -f \
    apps/rust-trading-bot/trader.log \
    apps/python-telegram-monitor/telegram_monitor.log
```

### 只看重要信息
```bash
tail -f apps/rust-trading-bot/trader.log | grep -E "📨|✅|❌|🚨"
```

---

## 🧪 测试信号流

### 运行集成测试
```bash
cd apps/python-telegram-monitor
bash test_integration.sh
```

### 手动发送测试信号
```bash
curl -X POST http://localhost:8080/api/signals \
    -H "Content-Type: application/json" \
    -d '{
        "symbol": "BTCUSDT",
        "side": "LONG",
        "entry_price": 95000,
        "stop_loss": 94000,
        "confidence": "HIGH",
        "leverage": 10,
        "source": "manual_test",
        "timestamp": '$(date +%s)',
        "raw_message": "手动测试"
    }'
```

---

## 🌐 Web界面

### 启动前端监控面板
```bash
cd apps/rust-trading-bot/web
npm run dev
```

访问: http://localhost:5173

---

## 🔧 常见问题

### Q: Rust引擎启动失败?
```bash
# 1. 检查端口占用
netstat -tlnp | grep 8080

# 2. 查看错误日志
tail -50 apps/rust-trading-bot/trader.log

# 3. 检查Binance API配置
cat .env | grep BINANCE
```

### Q: Python监控连接失败?
```bash
# 1. 检查Telegram配置
cat .env | grep TELEGRAM

# 2. 重新登录Telegram
cd apps/python-telegram-monitor
rm telegram_session.session
python3 telegram_monitor.py  # 输入验证码
```

### Q: 信号没有被处理?
```bash
# 1. 确认Rust引擎收到信号
grep "收到Telegram信号" apps/rust-trading-bot/trader.log

# 2. 查看数据库记录
curl http://localhost:8080/api/telegram-signals | jq '.[] | {symbol, side}'

# 3. 检查Python日志
grep "✅ 信号发送成功" apps/python-telegram-monitor/telegram_monitor.log
```

---

## 📁 重要文件位置

```
web3/
├── .env                                      # 配置文件 (API密钥等)
├── start_trading.sh                          # 启动脚本
├── stop_trading.sh                           # 停止脚本
├── QUICK_START.md                            # 本文档
├── HYBRID_ARCHITECTURE_CHECKLIST.md          # 完整验证清单
│
├── apps/rust-trading-bot/
│   ├── trader.log                            # Rust引擎日志
│   ├── trader.pid                            # Rust引擎PID
│   ├── data/trading.db                       # SQLite数据库
│   └── web/                                  # 前端面板
│
└── apps/python-telegram-monitor/
    ├── telegram_monitor.log                  # Python监控日志
    ├── monitor.pid                           # Python监控PID
    ├── telegram_session.session              # Telegram会话
    ├── README.md                             # Python模块文档
    ├── DEPLOYMENT.md                         # 部署指南
    └── test_integration.sh                   # 集成测试
```

---

## 🎯 启动后要做的事

1. **首次启动需要Telegram登录** (只需一次)
   - Python会提示输入验证码
   - 输入后会保存session文件

2. **检查系统状态**
   ```bash
   curl http://localhost:8080/api/status
   ```

3. **配置监控的Telegram频道**
   - 编辑 `.env` 文件
   - 设置 `TELEGRAM_CHANNELS=-1001234567890,@channel_name`
   - 重启Python监控

4. **运行集成测试**
   ```bash
   cd apps/python-telegram-monitor
   bash test_integration.sh
   ```

5. **观察日志1-2小时**
   ```bash
   tail -f apps/*//*.log
   ```

---

## 📞 获取帮助

- **详细部署指南**: `apps/python-telegram-monitor/DEPLOYMENT.md`
- **迁移报告**: `apps/python-telegram-monitor/MIGRATION_REPORT.md`
- **验证清单**: `HYBRID_ARCHITECTURE_CHECKLIST.md`
- **项目配置说明**: `.claude/CLAUDE.md`

---

**最后更新**: 2025-11-21
**维护者**: AI Trading System
