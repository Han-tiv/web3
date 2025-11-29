# Telegram交易信号监控

## 📋 项目介绍

这是一个基于Python的Telegram频道监控模块,负责监听交易信号频道的消息,解析交易信号,并通过HTTP API发送到Rust交易引擎执行。

### 🎯 设计理念

**混合架构**: Python监控 + Rust交易引擎
- **Python负责**: Telegram连接、消息监听、信号解析 (利用Telethon的稳定性)
- **Rust负责**: 订单执行、持仓管理、风险控制 (利用Rust的性能)

### ✅ 优势

1. **稳定性**: Telethon库成熟稳定,避免了grammers的连接问题
2. **低耦合**: Python和Rust通过HTTP API通信,独立部署和升级
3. **易维护**: Python代码简洁,调试方便,迭代快速
4. **高性能**: 关键的交易执行仍由Rust负责

---

## 📦 安装

### 1. 安装依赖

```bash
cd /home/hanins/code/web3/apps/python-telegram-monitor
pip install -r requirements.txt
```

### 2. 配置环境变量

在**项目根目录** `/home/hanins/code/web3/.env` 中添加:

```bash
# Telegram配置
TELEGRAM_API_ID=2040
TELEGRAM_API_HASH=b18441a1ff607e10a989891a5462e627
TELEGRAM_PHONE=+17578852234

# 监控的频道列表 (逗号分隔)
# 支持格式: 频道用户名(@channel)、频道ID、频道链接
TELEGRAM_CHANNELS=-1001234567890,-1009876543210

# Rust交易引擎地址
RUST_ENGINE_URL=http://localhost:8080
RUST_ENGINE_TIMEOUT=5

# 日志配置
LOG_LEVEL=INFO
LOG_FILE=telegram_monitor.log
```

### 3. 首次登录

```bash
python telegram_monitor.py
```

首次运行时需要输入Telegram验证码,会自动保存会话文件 `telegram_session.session`。

---

## 🚀 使用方法

### 启动监控

可选择两种运行模式:

- `signal_forwarder.py`：默认推荐的 valuescaner 专用转发器（精简解析、多冗余过滤、日志输出到 `telegram_forwarder.log`）
- `telegram_monitor.py`：兼容旧版的通用解析器（日志输出到 `telegram_monitor.log`）

```bash
# 前台运行（推荐转发器）
python signal_forwarder.py

# 兼容旧版通用监控
python telegram_monitor.py

# 使用启动脚本并指定模式（默认 signal_forwarder）
bash start_monitor.sh signal_forwarder

# 后台运行 (生产环境示例)
nohup python signal_forwarder.py > telegram_forwarder.log 2>&1 &
```

> 在根目录执行 `bash start_trading.sh` 时，会自动选择 `signal_forwarder.py` 作为默认 worker，并写入 `apps/python-telegram-monitor/telegram.pid`。

### 停止监控

```bash
# 找到进程
ps aux | grep telegram_monitor

# 停止进程
kill <PID>

# 或使用systemd
sudo systemctl stop telegram-monitor
```

### 查看日志

```bash
# 实时查看
tail -f telegram_monitor.log

# 搜索错误
grep ERROR telegram_monitor.log

# 查看统计
grep "运行统计" telegram_monitor.log
```

---

## 📊 功能特性

### 1. 信号解析

支持多种格式的交易信号:

```
✅ BTCUSDT LONG 95000 SL:94000 TP:96000
✅ ETH做多 入场:3500 止损:3400
✅ SOL/USDT 做空 @145.5 止损147 10X
✅ BNB 买入 600 SL:590
```

### 2. 信号去重

自动识别5分钟内的重复信号,避免重复下单。

### 3. 错误恢复

- 自动重连Telegram
- HTTP请求超时保护
- 详细的错误日志

### 4. 监控统计

每5分钟自动输出运行统计:
- 收到消息数
- 解析信号数
- 成功发送数
- 错误次数

---

## 🔧 配置说明

### 频道配置

在 `.env` 中配置 `TELEGRAM_CHANNELS`:

```bash
# 方式1: 使用频道ID (推荐)
TELEGRAM_CHANNELS=-1001234567890,-1009876543210

# 方式2: 使用频道用户名
TELEGRAM_CHANNELS=@trading_signals,@crypto_alerts

# 方式3: 混合使用
TELEGRAM_CHANNELS=-1001234567890,@trading_signals
```

### 获取频道ID

```bash
# 使用辅助脚本
python -c "
from telethon import TelegramClient
import asyncio

async def main():
    client = TelegramClient('temp', API_ID, API_HASH)
    await client.start()
    async for dialog in client.iter_dialogs():
        if dialog.is_channel:
            print(f'{dialog.name}: {dialog.id}')

asyncio.run(main())
"
```

---

## 🔗 与Rust引擎集成

### API接口

Python监控通过以下接口与Rust引擎通信:

```http
POST /api/signals
Content-Type: application/json

{
  "symbol": "BTCUSDT",
  "raw_message": "BTCUSDT LONG 95000 SL:94000 TP:96000",
  "timestamp": 1700000000.0
}
```

### 响应格式

```json
{
  "status": "received",
  "symbol": "BTCUSDT",
  "queued_at": "2024-11-20T10:00:00Z"
}
```

---

## 🧪 测试

### 测试信号解析

```bash
python signal_parser.py
```

### 测试配置

```bash
python config.py
```

### 手动测试完整流程

```bash
# 1. 启动Rust引擎
cd ../rust-trading-bot
cargo run --release --bin integrated_ai_trader

# 2. 启动Python监控
cd ../python-telegram-monitor
python telegram_monitor.py
```

---

## 📁 文件结构

```
python-telegram-monitor/
├── telegram_monitor.py     # 主程序
├── signal_parser.py        # 信号解析
├── config.py               # 配置管理
├── requirements.txt        # Python依赖
├── README.md               # 文档
└── telegram_session.session  # Telegram会话 (自动生成)
```

---

## ⚠️ 注意事项

1. **会话文件**: `telegram_session.session` 包含登录信息,请妥善保管
2. **频道权限**: 确保Telegram账号已加入要监听的频道
3. **API限流**: Telegram有API调用频率限制,建议监听频道数不超过10个
4. **网络稳定**: 需要稳定的网络连接到Telegram服务器

---

## 🔍 故障排查

### 1. 连接失败

```bash
# 检查网络
ping telegram.org

# 检查代理设置 (如需要)
export HTTP_PROXY=http://127.0.0.1:7890
export HTTPS_PROXY=http://127.0.0.1:7890
```

### 2. 无法接收消息

- 确认已加入频道
- 检查频道ID是否正确
- 查看日志中的频道列表

### 3. 信号解析失败

- 检查消息格式是否符合规则
- 运行 `python signal_parser.py` 测试

### 4. Rust引擎连接失败

```bash
# 检查Rust引擎是否运行
curl http://localhost:8080/health

# 检查端口
netstat -tlnp | grep 8080
```

---

## 📈 性能指标

- **消息处理延迟**: < 100ms
- **信号发送延迟**: < 50ms (本地通信)
- **内存占用**: ~50MB
- **CPU占用**: < 1%

---

## 🔄 升级计划

### 第2阶段: Redis队列

- 消息持久化
- 支持多实例
- 负载均衡

### 第3阶段: WebSocket

- 实时日志推送
- 监控面板集成

---

## 📞 支持

如有问题,请查看:
1. 日志文件: `telegram_monitor.log`
2. Rust引擎日志: `../rust-trading-bot/integrated_ai_trader.log`
3. 系统状态: `curl http://localhost:8080/health`
