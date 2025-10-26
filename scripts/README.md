# 🔧 Scripts Directory - 脚本目录

**所有项目脚本的统一管理**

---

## 📂 目录结构

```
scripts/
├── monitors/           # 监控启动脚本
├── deploy/             # 部署脚本
└── maintenance/        # 维护脚本
```

---

## 📋 脚本分类

### 1️⃣ monitors/ - 监控脚本

**用途**: 启动各种监控服务

| 脚本 | 说明 | 用法 |
|------|------|------|
| `start_6551_monitor.js` | 6551 频道监控 | `node scripts/monitors/start_6551_monitor.js` |
| `start_6551_kline_monitor.js` | 6551 K线数据监控 | `node scripts/monitors/start_6551_kline_monitor.js` |
| `start_tg_monitor.js` | Telegram 监控 | `node scripts/monitors/start_tg_monitor.js` |
| `start_all_monitors.js` | 启动所有监控 | `node scripts/monitors/start_all_monitors.js` |

**快速开始**:
```bash
# 启动所有监控
cd /home/hanins/code/web3
node scripts/monitors/start_all_monitors.js

# 或单独启动
node scripts/monitors/start_6551_monitor.js
```

---

### 2️⃣ deploy/ - 部署脚本

**用途**: 生产环境部署和启动

| 脚本 | 说明 | 用法 |
|------|------|------|
| `start.sh` | 主启动脚本 | `./scripts/deploy/start.sh` |

**快速开始**:
```bash
# 赋予执行权限
chmod +x scripts/deploy/start.sh

# 运行
./scripts/deploy/start.sh
```

---

### 3️⃣ maintenance/ - 维护脚本

**用途**: 日常维护、清理和优化

| 脚本 | 说明 | 用法 | 频率 |
|------|------|------|------|
| `weekly_cleanup.sh` | 每周清理 | `./scripts/maintenance/weekly_cleanup.sh` | 每周 |
| `security_check.sh` | 安全检查 | `./scripts/maintenance/security_check.sh` | 每天 |
| `prewarm-mcp.sh` | MCP 预热 | `./scripts/maintenance/prewarm-mcp.sh` | 启动时 |
| `database-optimization.sql` | 数据库优化 | `sqlite3 db.sqlite < scripts/maintenance/database-optimization.sql` | 每月 |

**快速开始**:
```bash
# 每周清理（删除旧日志、临时文件等）
./scripts/maintenance/weekly_cleanup.sh

# 安全检查（检查敏感文件、权限等）
./scripts/maintenance/security_check.sh
```

---

## 🚀 常用场景

### 场景 1: 开发环境启动

```bash
# 1. 启动监控服务
node scripts/monitors/start_all_monitors.js

# 2. 另开终端，启动交易机器人
cd apps/rust-trading-bot
cargo run --release --bin show_assets
```

### 场景 2: 生产环境部署

```bash
# 使用启动脚本
./scripts/deploy/start.sh
```

### 场景 3: 定期维护

```bash
# 设置 crontab
crontab -e

# 添加定时任务
0 2 * * 0 /home/hanins/code/web3/scripts/maintenance/weekly_cleanup.sh
0 9 * * * /home/hanins/code/web3/scripts/maintenance/security_check.sh
```

---

## 📝 开发规范

### 添加新脚本

1. **确定类别**
   - 监控相关 → `monitors/`
   - 部署相关 → `deploy/`
   - 维护相关 → `maintenance/`

2. **命名规范**
   - Shell 脚本: `小写_下划线.sh`
   - Node.js 脚本: `小写_下划线.js`
   - 描述性名称

3. **添加注释**
   ```bash
   #!/bin/bash
   # 脚本名称和用途
   # 作者: xxx
   # 日期: 2025-xx-xx
   
   # 使用说明
   # ./script_name.sh [options]
   ```

4. **更新文档**
   - 在本文件中添加说明
   - 更新相关使用文档

### 脚本模板

**Shell 脚本模板**:
```bash
#!/bin/bash
# 脚本描述
# 用途: xxx
# 执行: ./script_name.sh

set -e

PROJECT_ROOT="/home/hanins/code/web3"
cd "$PROJECT_ROOT"

echo "🔧 脚本开始执行..."
echo "═══════════════════════════════════════════"
echo ""

# 主要逻辑
# ...

echo "✅ 脚本执行完成！"
```

**Node.js 脚本模板**:
```javascript
#!/usr/bin/env node
/**
 * 脚本描述
 * 用途: xxx
 * 执行: node script_name.js
 */

const path = require('path');
const fs = require('fs');

const PROJECT_ROOT = '/home/hanins/code/web3';

console.log('🔧 脚本开始执行...');
console.log('═══════════════════════════════════════════\n');

// 主要逻辑
// ...

console.log('\n✅ 脚本执行完成！');
```

---

## 🔍 故障排除

### 常见问题

#### Q1: 脚本没有执行权限

```bash
chmod +x scripts/path/to/script.sh
```

#### Q2: Node.js 脚本找不到模块

```bash
# 确保在项目根目录运行
cd /home/hanins/code/web3
node scripts/monitors/start_xxx.js
```

#### Q3: Shell 脚本路径错误

检查脚本中的 `PROJECT_ROOT` 变量是否正确。

---

## 📊 脚本统计

```
总脚本数: 8 个
├── monitors:     4 个
├── deploy:       1 个
└── maintenance:  4 个 (3个 sh + 1个 sql)
```

---

## 🔗 相关文档

- [项目结构说明](../PROJECT_STRUCTURE.md)
- [部署指南](../docs/deployment/DEPLOYMENT_GUIDE.md)
- [维护手册](../docs/guides/LOGGING_STANDARD.md)

---

**🔧 统一管理，方便维护！**

_最后更新: 2025-10-26_
