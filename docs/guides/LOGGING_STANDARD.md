# 📝 统一日志标准

> **Linus式日志哲学**: "好的日志应该简洁、一致、可搜索"
> **重要提示**: Crypto Bot 模块已于 2025-02 下线，本文中的相关日志示例保留作为历史参考。

## 🎯 日志标准概述

### 统一格式规范
```
时间戳 [服务] [级别] 组件: 消息 {结构化数据}
```

**示例**:
```
2025-09-29T10:30:45.123Z [crypto-bot] [INFO] scheduler: Task processing started {task_id: "123", priority: 8}
2025-09-29T10:30:46.456Z [trading-engine] [ERROR] risk_manager: Daily loss limit exceeded {loss: 85, limit: 80}
2025-09-29T10:30:47.789Z [ai-predictor] [WARN] model: Low confidence prediction {symbol: "ETHUSDT", confidence: 0.62}
```

> ℹ️ **Legacy 提示**: `crypto-bot` 相关日志示例与命令仅供历史查阅，当前仓库已移除该模块。

## 🔧 语言实现标准

### Go (Zap替代标准log)

#### 当前问题
```go
// ❌ 当前: 使用标准log包
log.Println("调度器已启动")
log.Printf("Error: %v", err)
```

#### 推荐实现
```go
// ✅ 推荐: 使用uber-go/zap
package logger

import (
	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
)

var Logger *zap.Logger

func InitLogger(service string) error {
	config := zap.NewProductionConfig()
	config.EncoderConfig.TimeKey = "timestamp"
	config.EncoderConfig.EncodeTime = zapcore.ISO8601TimeEncoder
	config.InitialFields = map[string]interface{}{
		"service": service,
	}

	var err error
	Logger, err = config.Build()
	return err
}

// 使用示例
func (s *Scheduler) Start() {
	logger.Logger.Info("scheduler: Task processor started",
		zap.Int("interval_seconds", 30),
	)
}
```

#### 安装
```bash
cd apps/crypto-bot/backend
go get -u go.uber.org/zap
```

### Python (保持loguru，但统一配置)

#### 当前实现 ✅
```python
from loguru import logger

# ✅ 已使用loguru，只需统一配置
```

#### 标准配置
```python
# config/logging.py
from loguru import logger
import sys

def setup_logger(service_name: str, log_level: str = "INFO"):
    """统一日志配置"""
    # 移除默认handler
    logger.remove()

    # 控制台输出 (带颜色)
    logger.add(
        sys.stdout,
        format="<green>{time:YYYY-MM-DD HH:mm:ss.SSS}</green> | <level>[{level:8}]</level> | <cyan>[{extra[service]}]</cyan> | <level>{message}</level>",
        level=log_level,
        colorize=True,
    )

    # 文件输出 (JSON格式)
    logger.add(
        f"logs/{service_name}.log",
        format="{time:YYYY-MM-DD HH:mm:ss.SSS} [{level:8}] [{extra[service]}] {message}",
        level=log_level,
        rotation="10 MB",
        retention="7 days",
        compression="zip",
        serialize=False,  # 纯文本，便于grep
    )

    # 错误日志单独文件
    logger.add(
        f"logs/{service_name}_error.log",
        format="{time:YYYY-MM-DD HH:mm:ss.SSS} [{level:8}] [{extra[service]}] {message}",
        level="ERROR",
        rotation="5 MB",
        retention="14 days",
        backtrace=True,
        diagnose=True,
    )

    # 配置service名称
    logger.configure(extra={"service": service_name})

    return logger

# 使用示例
# main.py
from config.logging import setup_logger

logger = setup_logger("ai-predictor", log_level="INFO")
logger.info("AI prediction started", symbol="ETHUSDT", confidence=0.78)
```

### TypeScript/Node.js (保持Winston，但统一配置)

#### 当前实现 ✅
```typescript
// ✅ 已使用winston
```

#### 标准配置
```typescript
// src/config/logger.ts
import winston from 'winston';

const createLogger = (serviceName: string) => {
  const logFormat = winston.format.combine(
    winston.format.timestamp({ format: 'YYYY-MM-DD HH:mm:ss.SSS' }),
    winston.format.errors({ stack: true }),
    winston.format.printf(({ level, message, timestamp, service, ...meta }) => {
      const metaStr = Object.keys(meta).length > 0
        ? ` ${JSON.stringify(meta)}`
        : '';
      return `${timestamp} [${level.toUpperCase().padEnd(5)}] [${service}] ${message}${metaStr}`;
    })
  );

  return winston.createLogger({
    level: process.env.LOG_LEVEL || 'info',
    defaultMeta: { service: serviceName },
    format: logFormat,
    transports: [
      // 控制台输出 (带颜色)
      new winston.transports.Console({
        format: winston.format.combine(
          winston.format.colorize(),
          logFormat
        ),
      }),

      // 通用日志文件
      new winston.transports.File({
        filename: `logs/${serviceName}.log`,
        maxsize: 10 * 1024 * 1024, // 10MB
        maxFiles: 10,
        tailable: true,
      }),

      // 错误日志文件
      new winston.transports.File({
        filename: `logs/${serviceName}_error.log`,
        level: 'error',
        maxsize: 5 * 1024 * 1024, // 5MB
        maxFiles: 5,
      }),
    ],
  });
};

export const logger = createLogger(process.env.SERVICE_NAME || 'trading-engine');

// 使用示例
logger.info('Trading signal generated', {
  symbol: 'ETHUSDT',
  direction: 'LONG',
  confidence: 0.75
});
```

## 📊 日志级别标准

### 级别定义

| 级别 | 用途 | 示例 |
|------|------|------|
| **DEBUG** | 详细调试信息 | `logger.debug("API request details", {url, params})` |
| **INFO** | 正常操作信息 | `logger.info("Task completed successfully", {task_id})` |
| **WARN** | 警告但不影响运行 | `logger.warn("High volatility detected", {volatility: 0.08})` |
| **ERROR** | 错误需要关注 | `logger.error("Database connection failed", {error: err.message})` |
| **FATAL** | 严重错误导致退出 | `logger.fatal("Critical config missing", {config: "API_KEY"})` |

### 使用原则

#### ✅ 好的日志
```typescript
// 结构化、可搜索、包含上下文
logger.info('Order executed', {
  order_id: '12345',
  symbol: 'ETHUSDT',
  side: 'BUY',
  quantity: 1.5,
  price: 3500.0,
  execution_time_ms: 250
});
```

#### ❌ 坏的日志
```typescript
// 字符串拼接、难以解析
logger.info('Order 12345 executed: BUY 1.5 ETHUSDT @ 3500.0');
```

## 🔍 日志搜索和分析

### grep搜索示例
```bash
# 搜索特定级别
grep "\[ERROR\]" logs/trading-engine.log

# 搜索特定组件
grep "risk_manager:" logs/trading-engine.log

# 搜索特定字段
grep "task_id.*123" logs/crypto-bot.log

# 按时间范围搜索
grep "2025-09-29 10:3[0-9]" logs/*.log

# 多条件搜索
grep -E "\[ERROR\].*trading_engine.*ETHUSDT" logs/*.log
```

### 统计分析
```bash
# 错误数量统计
grep -c "\[ERROR\]" logs/trading-engine.log

# 每小时请求量
grep "2025-09-29" logs/api.log | cut -d' ' -f2 | cut -d':' -f1 | uniq -c

# 最常见的错误
grep "\[ERROR\]" logs/*.log | cut -d':' -f4- | sort | uniq -c | sort -rn | head -10
```

## 📁 日志文件组织

### 目录结构
```
logs/
├── crypto-bot.log          # Legacy: Crypto Bot 主日志 (模块已下线)
├── crypto-bot_error.log    # Legacy: Crypto Bot 错误日志 (保留历史)
├── trading-engine.log      # Trading引擎主日志
├── trading-engine_error.log
├── ai-predictor.log        # AI预测器主日志
├── ai-predictor_error.log
├── social-monitor.log      # 社交监控主日志
├── social-monitor_error.log
└── archive/                # 归档日志 (7天后)
    ├── crypto-bot.2025-09-22.log.gz  # Legacy 归档
    └── trading-engine.2025-09-22.log.gz
```

### 日志轮转配置

#### Logrotate (Linux)
```bash
# /etc/logrotate.d/web3-monorepo
/home/hanins/code/logs/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 0644 hanins hanins
    sharedscripts
    postrotate
        docker-compose restart > /dev/null 2>&1 || true
    endscript
}
```

## 🚀 迁移计划

### Phase 1: 准备 (完成)
- [x] 创建统一日志标准文档
- [x] 定义日志格式和级别

### Phase 2: 实施 (1周)

#### 1. Go服务迁移
```bash
# 1. 安装Zap
cd apps/crypto-bot/backend
go get -u go.uber.org/zap

# 2. 创建logger包
mkdir -p pkg/logger
# 复制本文档中的Go实现到 pkg/logger/logger.go

# 3. 逐个文件替换
# 替换: log.Println -> logger.Logger.Info
# 替换: log.Printf -> logger.Logger.Infof
# 替换: log.Fatal -> logger.Logger.Fatal
```

#### 2. Python服务迁移
```bash
# 1. 创建logging配置
mkdir -p apps/crypto-bot/collector/config
# 复制本文档中的Python实现到 config/logging.py

# 2. 更新main.py
# 替换: logger.info -> logger.bind(service="collector").info

# 3. 统一格式
# 确保所有logger.info都使用结构化参数
```

#### 3. TypeScript服务迁移
```bash
# 1. 更新logger配置
cd apps/kronos-defi/packages/trading-engine
# 更新 src/logger.ts 使用本文档中的配置

# 2. 确保所有服务使用统一配置
# trading-engine, web-dashboard, twitter-monitor
```

### Phase 3: 验证 (3天)
```bash
# 1. 启动所有服务
./start.sh

# 2. 检查日志格式
tail -f logs/*.log
# 验证格式一致性

# 3. 测试日志搜索
grep -E "\[ERROR\]" logs/*.log
# 验证可搜索性

# 4. 验证日志轮转
# 手动触发轮转测试
```

## 📈 监控集成

### Prometheus日志指标
```go
// Go示例: 导出日志指标
import "github.com/prometheus/client_golang/prometheus"

var (
	logCounter = prometheus.NewCounterVec(
		prometheus.CounterOpts{
			Name: "log_messages_total",
			Help: "Total number of log messages",
		},
		[]string{"service", "level"},
	)
)

// 在logger中增加计数
func logWithMetrics(level, service, message string) {
	logger.Logger.Info(message, zap.String("service", service))
	logCounter.WithLabelValues(service, level).Inc()
}
```

### Grafana日志查询
```promql
# 错误率趋势
rate(log_messages_total{level="ERROR"}[5m])

# 服务日志量对比
sum by (service) (log_messages_total)

# 错误日志告警
sum(rate(log_messages_total{level="ERROR"}[1m])) > 10
```

## 🔐 敏感信息处理

### 日志脱敏规则

```typescript
// 自动脱敏敏感字段
const sensitiveFields = ['password', 'api_key', 'secret', 'token', 'private_key'];

function sanitize(obj: any): any {
  const sanitized = { ...obj };
  for (const key of Object.keys(sanitized)) {
    if (sensitiveFields.some(field => key.toLowerCase().includes(field))) {
      sanitized[key] = '***REDACTED***';
    }
  }
  return sanitized;
}

// 使用
logger.info('User login', sanitize(userData));
```

```python
# Python脱敏
import re

def sanitize_log_message(message: str) -> str:
    """脱敏敏感信息"""
    # 脱敏API密钥
    message = re.sub(r'(api[_-]?key["\']?\s*[:=]\s*["\']?)[\w-]+', r'\1***REDACTED***', message, flags=re.IGNORECASE)
    # 脱敏密码
    message = re.sub(r'(password["\']?\s*[:=]\s*["\']?)[\w-]+', r'\1***REDACTED***', message, flags=re.IGNORECASE)
    return message

logger.info(sanitize_log_message(f"Config: {config}"))
```

```go
// Go脱敏
func sanitizeMessage(msg string) string {
	// 脱敏API密钥
	apiKeyRe := regexp.MustCompile(`(?i)(api[_-]?key["']?\s*[:=]\s*["']?)[\w-]+`)
	msg = apiKeyRe.ReplaceAllString(msg, "${1}***REDACTED***")

	// 脱敏密码
	passwordRe := regexp.MustCompile(`(?i)(password["']?\s*[:=]\s*["']?)[\w-]+`)
	msg = passwordRe.ReplaceAllString(msg, "${1}***REDACTED***")

	return msg
}
```

## 🛠️ 开发工具

### 实时日志查看
```bash
# 所有服务日志
tail -f logs/*.log | grep -E --line-buffered --color=always '\[(ERROR|WARN)\]'

# 特定服务
tail -f logs/trading-engine.log

# 多服务彩色输出
tail -f logs/crypto-bot.log | sed 's/^/[BOT] /' & \
tail -f logs/trading-engine.log | sed 's/^/[TRADING] /' & \
tail -f logs/ai-predictor.log | sed 's/^/[AI] /'
```

### 日志分析工具
```bash
# 安装lnav (日志分析工具)
brew install lnav  # macOS
sudo apt install lnav  # Linux

# 使用lnav查看日志
lnav logs/*.log
```

## 📞 故障排除

### 日志文件权限
```bash
# 确保logs目录可写
mkdir -p logs
chmod 755 logs
chown -R $USER:$USER logs
```

### 日志目录不存在
```bash
# 启动脚本中自动创建
#!/bin/bash
mkdir -p logs
./start_services.sh
```

### 日志文件过大
```bash
# 手动清理旧日志
find logs/ -name "*.log" -mtime +7 -delete
find logs/ -name "*.log.gz" -mtime +30 -delete

# 或使用logrotate (推荐)
```

## 💡 最佳实践总结

### ✅ 推荐做法
1. **结构化日志**: 使用JSON或键值对，不要字符串拼接
2. **包含上下文**: 每条日志包含足够的排查信息（IDs, 时间戳, 参数）
3. **合理级别**: INFO用于正常流程，WARN用于异常但可继续，ERROR用于需要处理的错误
4. **敏感信息脱敏**: 永远不要记录密码、API密钥、私钥
5. **适度日志**: 不要在循环中打DEBUG日志

### ❌ 避免做法
1. **过度日志**: 不要记录每个变量的值
2. **日志污染**: 不要用日志调试后忘记删除
3. **阻塞日志**: 不要使用同步写入影响性能
4. **忽略错误**: 不要吞掉异常不记录
5. **多余日志**: 不要重复记录相同信息

---

## 🎯 预期效果

实施统一日志标准后：

✅ **可搜索性**: 一条grep命令快速定位问题
✅ **一致性**: 所有服务日志格式统一
✅ **可分析性**: 方便统计和趋势分析
✅ **可维护性**: 团队成员快速理解日志
✅ **安全性**: 敏感信息自动脱敏

---

**这是"好品味"的日志系统: 简洁、统一、可搜索。**
