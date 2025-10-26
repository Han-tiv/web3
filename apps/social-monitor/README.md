# 🌐 Social Media Monitor

**统一的社交媒体监控平台，专注于Web3和加密货币机会发现**

## 🎯 功能概述

Social Monitor 是一个综合性的社交媒体监控系统，集成了多个平台的数据收集和分析能力：

- **🐦 Twitter监控** (通过Nitter) - 隐私友好的Twitter内容监控
- **📱 Telegram监控** - 实时监控Telegram群组和频道
- **💬 Discord监控** - Discord服务器消息监控
- **🔄 数据聚合** - 统一的数据处理和API服务
- **📊 实时分析** - WebSocket实时数据推送

## 🏗️ 架构设计

```
social-monitor/
├── services/
│   ├── nitter/           # Twitter监控服务 (Nitter实例)
│   ├── telegram/         # Telegram监控服务
│   ├── discord/          # Discord监控服务
│   └── aggregator/       # 数据聚合和API服务
├── config/               # 配置文件
├── data/                 # 数据存储
├── logs/                 # 日志文件
└── scripts/             # 运维脚本
```

## 🚀 快速启动

### 环境要求
- Docker & Docker Compose
- Node.js 18+
- Redis (通过Docker提供)

### 配置环境变量

```bash
# 复制环境变量模板
cp .env.example .env

# 编辑配置文件
nano .env
```

**必需配置**:
```env
# Telegram配置
TELEGRAM_API_ID=your_api_id
TELEGRAM_API_HASH=your_api_hash
TELEGRAM_BOT_TOKEN=your_bot_token
TELEGRAM_CHAT_ID=your_chat_id

# ValueScan 凭证（异动推送）
VALUESCAN_BEARER_TOKEN=your_valuescan_bearer
VALUESCAN_ACCESS_TICKET=your_valuescan_ticket

# Discord配置
DISCORD_BOT_TOKEN=your_discord_bot_token

# 监控目标
TWITTER_USERS=binance,coinbase,elonmusk
TELEGRAM_GROUPS=@cryptogroup1,@cryptogroup2
DISCORD_SERVERS=server_id_1,server_id_2
```

> 可选参数：`VALUESCAN_CRON`（默认 `*/2 * * * *`，每2分钟轮询）、`VALUESCAN_MIN_TRIGGERS_24H`（最小小周期异动次数过滤）、`TELEGRAM_DRY_RUN=true`（仅记录日志不推送）、`TELEGRAM_SILENT=true`（静默发送）。

### 启动服务

```bash
# 启动所有服务
npm run start

# 查看日志
npm run logs

# 检查状态
npm run status
```

### 开发模式

```bash
# 启动开发环境
npm run dev

# 单独启动服务
npm run nitter:dev
npm run telegram:dev
npm run discord:dev
npm run aggregator:dev
```

## 📡 服务端口

| 服务 | 端口 | 描述 |
|------|------|------|
| Nitter | 8080 | Twitter界面和API |
| Aggregator | 3002 | 数据聚合API和WebSocket |
| Dashboard | 3003 | 监控面板 |
| Redis | 6379 | 内部缓存 |

## 🔧 API接口

### 获取所有机会
```bash
GET /api/opportunities?type=airdrop&minPriority=7&limit=20
```

### 获取高优先级机会
```bash
GET /api/opportunities/priority
```

### 获取实时统计
```bash
GET /api/stats/realtime
```

### WebSocket连接
```javascript
const ws = new WebSocket('ws://localhost:3002');

ws.on('message', (data) => {
  const event = JSON.parse(data);
  console.log('收到事件:', event.type, event.data);
});
```

## 🚨 ValueScan 异动推送

- 后台任务每 `VALUESCAN_CRON` 设定周期调用 ValueScan `getFundsMovementPage`，筛选带有 `alpha` / `fomo` 标签的资金异动。
- 首次命中会通过 Telegram Bot (`TELEGRAM_BOT_TOKEN` + `TELEGRAM_CHAT_ID`) 推送提示，Redis 集合 `valuescan:funds:alerted` 负责 24 小时去重。
- 可通过 `POST /api/valuescan/scan` 手动触发一次抓取，便于调试或手动复核。
- 支持 `VALUESCAN_MIN_TRIGGERS_24H` 阈值过滤小周期异动次数，`TELEGRAM_DRY_RUN=true` 时仅记录日志不推送。

## 💡 机会类型识别

系统自动识别以下类型的加密货币机会：

- **🪂 Airdrop** - 空投活动
- **🎁 Giveaway** - 抽奖活动
- **🧧 RedPacket** - 红包活动
- **📚 Learn-to-Earn** - 学习赚钱项目
- **🎮 P2E** - 玩赚游戏

## 🔍 智能过滤

### 关键词检测
- 空投、airdrop、giveaway
- 红包、red packet
- 学习、learn、education
- 游戏、game、play-to-earn

### 优先级计算
```typescript
priority = basePriority + valueBonus + urgencyBonus + credibilityBonus
```

- **基础优先级** (1-5): 根据机会类型
- **价值加成** (0-3): 根据估算收益
- **紧急度加成** (0-2): 根据截止时间
- **可信度加成** (0-2): 根据来源质量

### 垃圾信息过滤
- 黑名单关键词过滤
- 重复内容检测
- 来源可信度评分
- 时间有效性验证

## 📊 监控面板

访问 `http://localhost:3003` 查看实时监控面板：

- **📈 实时统计** - 机会数量、类型分布
- **🎯 高价值机会** - 优先级排序
- **💹 收益分析** - 预期收益统计
- **🔄 服务状态** - 各服务健康状态
- **📱 实时通知** - WebSocket实时推送

## 🛠️ 运维命令

```bash
# 健康检查
npm run health

# 数据备份
npm run backup

# 清理日志
npm run clean

# 重启服务
npm run restart

# 查看服务状态
docker-compose ps
```

## 🔒 安全配置

### API访问控制
```env
JWT_SECRET=your_jwt_secret
API_KEY=your_api_key
```

### Telegram安全
- 使用Bot Token而非用户Token
- 限制群组访问权限
- 定期轮换API密钥

### Discord安全
- 最小权限原则
- 仅监听指定频道
- 启用消息内容意图

## 📝 日志管理

日志文件位置：
```
logs/
├── aggregator.log          # 聚合服务日志
├── telegram.log            # Telegram监控日志
├── discord.log             # Discord监控日志
├── nitter/                 # Nitter服务日志
└── redis/                  # Redis日志
```

日志级别：`error`, `warn`, `info`, `debug`

## 🚨 故障排除

### 常见问题

**1. Telegram连接失败**
```bash
# 检查API配置
echo $TELEGRAM_BOT_TOKEN
echo $TELEGRAM_API_ID

# 重新获取session
npm run telegram:auth
```

**2. Discord权限不足**
```bash
# 检查Bot权限
# 需要：读取消息历史、发送消息、使用斜杠命令
```

**3. Nitter无法访问Twitter**
```bash
# 检查代理设置
# 更新nitter.conf中的proxy配置
```

**4. Redis连接异常**
```bash
# 检查Redis状态
docker-compose logs redis

# 重启Redis
docker-compose restart redis
```

## 📈 性能优化

### Redis优化
```conf
maxmemory 256mb
maxmemory-policy allkeys-lru
save 60 1
```

### 监控频率调整
```env
# 降低监控频率以减少资源消耗
MONITOR_INTERVAL=30  # 30秒检查一次
CLEANUP_INTERVAL=3600  # 1小时清理一次
```

### 数据保留策略
- 机会数据：24小时
- 统计数据：7天
- 日志文件：30天

## 🤝 集成说明

### 与Crypto Bot集成

Social Monitor与Crypto Bot无缝集成：

```javascript
// 监听社交媒体机会
redis.subscribe('new_opportunity', (opportunity) => {
  // 转发给Crypto Bot处理
  cryptoBot.processOpportunity(opportunity);
});
```

### 外部系统集成

支持通过WebHook推送数据：

```bash
POST /webhook/opportunity
{
  "type": "new_opportunity",
  "data": {...}
}
```

## 📖 开发指南

### 添加新的监控源

1. 在 `services/` 下创建新目录
2. 实现数据收集逻辑
3. 连接到Redis发布订阅
4. 更新Docker Compose配置

### 自定义过滤规则

编辑 `config/filters.json`：

```json
{
  "keywords": {
    "high_priority": ["exclusive", "limited"],
    "blacklist": ["spam", "scam"]
  },
  "sources": {
    "trusted": ["@binance", "@coinbase"],
    "blocked": ["@suspicious_account"]
  }
}
```

---

**🔥 由Web3团队精心打造，为加密货币社区提供专业的社交媒体监控解决方案**
