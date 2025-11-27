# 🚀 RTB Telegram信号系统 - 快速启动指南

## ⚡ 一键启动

```bash
# 步骤1: 启动后端 (新终端)
cd /home/hanins/code/web3/apps/rust-trading-bot
./start_rtb.sh

# 步骤2: 启动前端 (新终端)
cd /home/hanins/code/web3/apps/rust-trading-bot/web
npm run dev

# 步骤3: 打开浏览器
# http://localhost:5173/telegram-signals
```

---

## 📊 系统架构

```
Telegram频道
    ↓
📱 integrated_ai_trader (Rust)
    ↓
🧠 SignalAnalyzer (关键词评分)
    ↓
💾 SQLite (telegram_signals表)
    ↓
🌐 Web API (Port 8080)
    ↓
🎨 React UI (Port 5173)
```

---

## 🎯 访问地址

| 服务 | URL |
|------|-----|
| 前端 | http://localhost:5173/telegram-signals |
| API | http://localhost:8080/api/telegram-signals |
| 健康检查 | http://localhost:8080/health |

---

## 📋 信号评分速查

| 评分 | 类型 | 图标 | 建议 |
|------|------|------|------|
| +5~+10 | 强烈看多 | 🔥🔥 | BUY |
| +3~+4 | 看多 | 📈 | BUY |
| +1~+2 | 中性偏多 | ➡️ | WATCH |
| 0 | 中性 | ➡️ | WATCH |
| -1~-2 | 中性偏空 | 📉 | WATCH |
| -3~-4 | 看空 | 📉 | AVOID |
| -5~-21 | 强烈看空 | 🚨 | CLOSE |

---

## 🔧 快速测试

```bash
# 测试API
curl http://localhost:8080/api/telegram-signals | jq .

# 查看数据库
sqlite3 data/trading.db "SELECT * FROM telegram_signals LIMIT 5;"

# 插入测试数据
sqlite3 data/trading.db "INSERT INTO telegram_signals (symbol, signal_type, score, keywords, recommend_action, reason, raw_message, timestamp) VALUES ('BTCUSDT', '强烈看多', 6, '+持续流入, +Alpha', 'BUY', '多个积极信号叠加', '测试消息', datetime('now'));"
```

---

## 🐛 常见问题

### API返回空数组？
- 等待Telegram消息到来
- 或插入测试数据 (见上方命令)

### 前端无法连接API？
- 检查后端是否运行: `curl http://localhost:8080/health`
- 检查端口占用: `lsof -i :8080`

### 编译错误？
```bash
cargo clean
cargo build --bin integrated_ai_trader --release
```

---

## 📚 完整文档

- `FINAL_SUMMARY.md` - 详细总结报告
- `RTB_TELEGRAM_INTEGRATION.md` - 技术文档
- `INTEGRATION_COMPLETE.md` - 集成验证

---

**版本**: v1.0.0 | **状态**: 生产就绪 ✅
