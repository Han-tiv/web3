# 🐦 Nitter API 前端仪表板

## 🚀 快速开始

### 1. 启动服务
```bash
cd /home/hantiv/code/Web3/apps/social-monitor/services/nitter
npm run build    # 构建TypeScript
npm run api      # 启动API服务器
```

### 2. 访问仪表板
打开浏览器访问：**http://localhost:3001/dashboard**

## 📊 功能特性

### 实时监控面板
- **服务状态监控**: 实时显示Nitter服务运行状态
- **统计数据展示**: 处理推文数、发现机会数、运行时长等
- **过滤器配置**: 查看账户分层、规则配置、反诈骗模式
- **最新机会**: 显示最近检测到的加密货币机会

### 智能过滤系统
- **账户分层**:
  - 🥇 Premium (14个账户): Binance, Coinbase等顶级交易所 (2.0x优先级)
  - 🥈 Verified (10个账户): 主流交易所和新闻媒体 (1.5x优先级)
  - 🥉 Standard (7个账户): 加密货币意见领袖 (1.2x优先级)

- **机会类型检测**:
  - 🧧 红包检测 (优先级9-10): 自动识别币安红包代码
  - 🪂 空投分析 (优先级7): 代币空投机会
  - 🎁 抽奖活动 (优先级6): 竞赛和彩票
  - 💡 Alpha信号 (优先级4-5): 交易机会和学习奖励

- **安全保护**:
  - 9种诈骗模式检测
  - 自动过滤可疑内容
  - 时效性验证

## 🌐 API端点

| 端点 | 方法 | 描述 |
|------|------|------|
| `/dashboard` | GET | 前端仪表板界面 |
| `/health` | GET | 服务健康状态检查 |
| `/stats` | GET | 运行统计信息 |
| `/opportunities` | GET | 最新检测到的机会 |
| `/filters` | GET | 过滤器配置信息 |
| `/accounts` | GET | 监控账户列表 |
| `/keywords` | GET | 监控关键词列表 |

## 🔧 测试工具

### 快速状态检查
```bash
./test_api.sh
```

### 手动API测试
```bash
# 健康检查
curl http://localhost:3001/health

# 获取统计
curl http://localhost:3001/stats

# 查看最新机会
curl http://localhost:3001/opportunities?limit=5

# 过滤器配置
curl http://localhost:3001/filters
```

### 过滤器单元测试
```bash
npm run test
```

## 📈 仪表板界面说明

### 主要区域
1. **服务状态卡片**: 显示连接状态和基本信息
2. **运行统计卡片**: 处理数据、发现机会等关键指标
3. **过滤配置卡片**: 规则数量、账户分层信息
4. **最新机会卡片**: 实时显示检测到的加密机会
5. **监控详情**: 完整的账户列表和关键词配置
6. **实时日志**: 服务运行日志流

### 交互功能
- **🔄 刷新所有数据**: 手动刷新所有统计信息
- **🔗 测试连接**: 验证API服务连接状态
- **🗑️ 清空显示**: 清除当前显示内容
- **自动刷新**: 每30秒自动更新数据

### 状态指示器
- 🟢 **在线**: 服务正常运行
- 🟡 **加载**: 正在检查或加载数据
- 🔴 **离线**: 服务不可用或连接失败

## 🛠️ 故障排除

### 常见问题

**1. 无法访问仪表板**
```bash
# 检查服务是否运行
lsof -i :3001
# 检查日志
npm run docker:logs
```

**2. Redis连接失败**
```bash
# 启动Redis容器
npm run docker:up
# 检查Redis状态
docker ps | grep redis
```

**3. 权限问题**
```bash
# 确保日志目录权限
mkdir -p logs
chmod 755 logs
```

**4. 端口被占用**
```bash
# 查找占用进程
lsof -i :3001
# 杀死进程
kill -9 <PID>
```

## 📊 运行数据示例

成功运行时的统计数据：
```json
{
  "service": "nitter",
  "uptime": 3600,
  "stats": {
    "tweetsProcessed": 150,
    "opportunitiesFound": 12,
    "startTime": "2025-09-25T03:58:33.754Z"
  },
  "monitoredAccounts": 30,
  "keywords": 19,
  "filters": {
    "totalRules": 6,
    "accountTiers": [...],
    "suspiciousPatternsCount": 9
  }
}
```

## 🔐 安全说明

- API服务器已配置CORS，支持跨域访问
- 所有敏感数据通过环境变量配置
- 内置反诈骗检测，自动过滤恶意内容
- 支持速率限制和错误处理

## 📱 移动端适配

仪表板完全响应式设计，支持：
- 📱 手机端浏览
- 📟 平板端显示
- 💻 桌面端全功能

立即打开 **http://localhost:3001/dashboard** 开始监控！