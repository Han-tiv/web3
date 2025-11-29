# 🚀 Valuescan V2 系统运行状态报告

**生成时间**: $(date '+%Y-%m-%d %H:%M:%S')
**系统版本**: Valuescan V2
**会话ID**: $(date '+%Y%m%d_%H%M%S')

---

## ✅ 系统启动状态

### 进程信息
- **PID**: $(cat trader.pid 2>/dev/null || echo "未找到PID文件")
- **进程状态**: $(ps -p $(cat trader.pid 2>/dev/null) > /dev/null 2>&1 && echo "✅ 运行中" || echo "❌ 未运行")
- **启动时间**: $(ps -p $(cat trader.pid 2>/dev/null) -o lstart= 2>/dev/null || echo "未知")
- **运行时长**: $(ps -p $(cat trader.pid 2>/dev/null) -o etime= 2>/dev/null || echo "未知")

### 环境变量
- **USE_VALUESCAN_V2**: $(grep -q "USE_VALUESCAN_V2=true" <(env) && echo "true (V2启用)" || echo "未设置或false")

---

## 📊 Telegram频道状态

### 当前配置

### 最近消息 (过去1小时)

📬 **过去1小时共 0 条消息**

---

## 🔍 V2版本验证

### 日志检查
```bash
# 检查Valuescan版本标识

# 检查V2评分信息

# 检查V2关键位信息
```

---

## 📈 系统运行数据

### Web API服务
- **地址**: http://localhost:8080
- **健康检查**: http://localhost:8080/health
- **状态**: $(curl -s http://localhost:8080/health 2>/dev/null | grep -q "ok" && echo "✅ 正常" || echo "⚠️  无响应")

### 账户信息
```json
```

### 当前持仓
```json
```

---

## 📋 最近日志 (最后50行)

```log
    
[2025-11-21T16:29:27Z INFO  integrated_ai_trader] 🔄 连接到 Telegram...
[2025-11-21T16:29:27Z INFO  grammers_client::client::net] creating a new sender with existing auth key to dc 1 149.154.175.53:443
[2025-11-21T16:29:27Z INFO  grammers_mtsender] connecting...
[2025-11-21T16:29:27Z INFO  grammers_mtproto::mtp::encrypted] got bad salt; salts have been reset down to a single one
[2025-11-21T16:29:27Z INFO  grammers_mtsender] incorrect server salt; re-sending request MsgId(7575216645119854616)
[2025-11-21T16:29:27Z INFO  grammers_mtproto::mtp::encrypted] only one future salt remaining; asking for more salts
[2025-11-21T16:29:27Z INFO  grammers_mtproto::mtp::encrypted] got 64 future salts
[2025-11-21T16:29:27Z INFO  grammers_mtsender] got rpc result MsgId(7575216645908473496) but no such request is saved
[2025-11-21T16:29:28Z INFO  integrated_ai_trader] ✅ Telegram已连接
    
[2025-11-21T16:29:28Z INFO  integrated_ai_trader] ✅ Binance客户端已初始化
    
[2025-11-21T16:29:28Z INFO  integrated_ai_trader] 📁 初始化数据库: data/trading.db
[2025-11-21T16:29:28Z INFO  integrated_ai_trader] ✅ 数据库已初始化
    
[2025-11-21T16:29:28Z INFO  integrated_ai_trader] 🔄 正在恢复启动前已存在的持仓...
[2025-11-21T16:29:28Z INFO  integrated_ai_trader] 📊 共恢复 0 个历史持仓
[2025-11-21T16:29:28Z INFO  integrated_ai_trader] ✅ 持仓监控线程已启动
    
[2025-11-21T16:29:28Z INFO  integrated_ai_trader] ✅ 延迟开仓队列重新分析线程已启动（每10分钟）
    
[2025-11-21T16:29:28Z INFO  integrated_ai_trader] ✅ Telegram健康监控线程已启动
    
[2025-11-21T16:29:28Z INFO  integrated_ai_trader] ✅ 初始合约余额（固定）: 50.03 USDT
[2025-11-21T16:29:28Z INFO  integrated_ai_trader] ✅ Web 服务器已启动 (端口 8080)
    
[2025-11-21T16:29:28Z INFO  integrated_ai_trader] 🔍 正在缓存所有频道实体...
[2025-11-21T16:29:28Z INFO  integrated_ai_trader] 🔍 持仓监控线程已启动
[2025-11-21T16:29:28Z INFO  integrated_ai_trader] 🔄 延迟开仓队列重新分析线程已启动
[2025-11-21T16:29:28Z INFO  integrated_ai_trader] 🔍 Telegram健康监控线程已启动
[2025-11-21T16:29:28Z INFO  rust_trading_bot::web_server] 🌐 Web API服务器启动: http://localhost:8080
[2025-11-21T16:29:28Z INFO  rust_trading_bot::web_server]    - 账户信息: http://localhost:8080/api/account
[2025-11-21T16:29:28Z INFO  rust_trading_bot::web_server]    - 权益历史: http://localhost:8080/api/equity-history
[2025-11-21T16:29:28Z INFO  rust_trading_bot::web_server]    - 当前持仓: http://localhost:8080/api/positions
[2025-11-21T16:29:28Z INFO  rust_trading_bot::web_server]    - 交易历史: http://localhost:8080/api/trades
[2025-11-21T16:29:28Z INFO  rust_trading_bot::web_server]    - 系统状态: http://localhost:8080/api/status
[2025-11-21T16:29:28Z INFO  rust_trading_bot::web_server]    - AI分析历史: http://localhost:8080/api/ai-history
[2025-11-21T16:29:28Z INFO  integrated_ai_trader] ✅ 目标频道已解析: valuescan (ID: 2254462672)
[2025-11-21T16:29:28Z INFO  integrated_ai_trader] ✅ 已缓存 1 个频道实体 (防止消息丢失)
[2025-11-21T16:29:28Z INFO  integrated_ai_trader] 📡 开始实时监控...
[2025-11-21T16:29:28Z INFO  integrated_ai_trader] ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    
[2025-11-21T16:30:52Z INFO  integrated_ai_trader] 
    📊 资金流入: PAXG 💰
[2025-11-21T16:30:52Z INFO  integrated_ai_trader]    价格: $4068.0000 | 24H: -0.19% | 类型: 合约
[2025-11-21T16:30:52Z INFO  integrated_ai_trader] 📡 Telegram信号: PAXGUSDT 评分:2 类型:中性偏多
[2025-11-21T16:30:52Z INFO  integrated_ai_trader] ⏭️ 跳过高价币种: PAXG ($4068.00), 价格>=1000
[2025-11-21T16:30:53Z INFO  rust_trading_bot::binance_client] 合约余额: 50.97434993 USDT
[2025-11-21T16:30:53Z INFO  rust_trading_bot::binance_client] 未实现盈亏: 0.00000000 USDT
```

---

## 🎯 V2特性清单

### ✅ 已实施的功能

1. **评分系统** (0-10分)
   - 关键位判断: 50% 权重
   - 资金流向: 30% 权重
   - 技术指标: 20% 权重
   - ≥6分才开仓

2. **开仓检查清单** (10项, 8项满足)
   - 距关键位>3%
   - 突破且量>1.5倍
   - 资金与价格一致
   - 止损≤5%
   - 风险收益比≥2:1
   - 单笔风险≤5%
   - 无FOMO/恐慌
   - 避开整数关口
   - 空间>3-5%
   - 最大亏损可承受

3. **持仓管理优先级**
   - 关键位止盈: 60% (最高优先级)
   - K线反转信号: 30%
   - 盈利时间参考: 10%

4. **代码自动止损**
   - 持仓>4h且盈利<1% → 自动全平
   - 亏损>-5% → 自动全平
   - 跌破Level 3支撑 → 自动全平

---

## 🔧 命令速查

### 查看实时日志
```bash
tail -f trader.log
```

### 查看V2评分
```bash
grep "V2评分" trader.log | tail -10
```

### 查看V2关键位
```bash
grep "V2关键位" trader.log | tail -10
```

### 停止系统
```bash
bash stop_trader.sh
# 或
kill $(cat trader.pid)
```

### 重启系统
```bash
bash stop_trader.sh && bash start_trader_v2.sh v2
```

---

## ⚠️  测试注意事项

1. **当前状态**: 系统已启动,等待频道发布新信号
2. **V2验证**: 需等待新信号产生,观察日志中的"V2评分"和"V2关键位"
3. **风险控制**: 建议小资金测试,单笔≤总资金5%
4. **实时监控**: 密切关注日志输出和持仓变化

---

**报告结束**
