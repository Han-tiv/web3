# 📁 Web3 Project - 项目结构说明

**最后更新**: 2025-10-26  
**版本**: v2.0 (重构后)

---

## 🎯 项目结构总览

```
Web3/
├── apps/                       # 应用程序
│   ├── rust-trading-bot/       # Rust 交易机器人 (主项目)
│   ├── social-monitor/         # 社交媒体监控
│   └── ds/                     # DeepSeek Python (已废弃)
│
├── scripts/                    # 脚本集合
│   ├── monitors/               # 监控启动脚本
│   ├── deploy/                 # 部署脚本
│   └── maintenance/            # 维护脚本
│
├── config/                     # 配置文件
│   ├── docker/                 # Docker 配置
│   └── environment/            # 环境配置
│
├── docs/                       # 文档中心
│   ├── architecture/           # 架构文档
│   ├── security/               # 安全文档
│   ├── optimization/           # 优化文档
│   ├── deployment/             # 部署文档
│   └── guides/                 # 使用指南
│
├── logs/                       # 日志文件 (gitignored)
├── .archive/                   # 归档文件 (gitignored)
│   ├── scripts/                # 历史脚本
│   ├── venv/                   # Python 虚拟环境
│   └── .codex/                 # Codex 缓存
│
├── node_modules/               # Node.js 依赖 (gitignored)
│
├── .env                        # 环境变量 (gitignored)
├── .env.example                # 环境变量模板
├── .gitignore                  # Git 忽略配置
├── package.json                # Node.js 项目配置
├── package-lock.json           # 依赖锁定文件
└── README.md                   # 项目主页
```

---

## 📦 目录说明

### 1. `apps/` - 应用程序

所有子项目的根目录。

#### apps/rust-trading-bot/
**主要项目**：高性能 Rust 交易机器人

```
rust-trading-bot/
├── src/                        # 源代码
│   ├── bin/                    # 可执行程序
│   │   ├── show_assets.rs      # 资产查询
│   │   ├── signal_trader.rs    # 信号交易
│   │   └── deepseek_trader.rs  # AI 交易 (新增)
│   ├── deepseek_client.rs      # DeepSeek API
│   ├── technical_analysis.rs   # 技术指标
│   ├── market_sentiment.rs     # 市场情绪
│   └── ...其他模块
│
├── docs/                       # 文档
│   ├── user-guide/             # 用户指南
│   ├── technical/              # 技术文档
│   ├── optimization/           # 优化报告
│   └── deepseek/               # DeepSeek AI 文档
│
├── scripts/                    # 脚本
│   └── run_deepseek_trader.sh  # 启动脚本
│
├── Cargo.toml                  # Rust 项目配置
├── Cargo.lock                  # 依赖锁定
└── README.md                   # 项目说明
```

#### apps/social-monitor/
**社交监控**：Twitter 信号监控系统

```
social-monitor/
├── services/
│   ├── nitter/                 # Twitter 监控
│   └── telegram/               # Telegram 监控
├── docs/                       # 文档
└── README.md
```

#### apps/ds/
**已废弃**：Python 版 DeepSeek 交易机器人（已迁移到 Rust）

---

### 2. `scripts/` - 脚本集合

所有项目脚本的统一管理目录。

#### scripts/monitors/
**监控脚本**：启动各种监控服务

- `start_6551_monitor.js` - 6551 频道监控
- `start_6551_kline_monitor.js` - 6551 K线监控
- `start_tg_monitor.js` - Telegram 监控
- `start_all_monitors.js` - 启动所有监控

**使用方法**:
```bash
cd /home/hanins/code/web3
node scripts/monitors/start_all_monitors.js
```

#### scripts/deploy/
**部署脚本**：生产环境部署

- `start.sh` - 主启动脚本

**使用方法**:
```bash
chmod +x scripts/deploy/start.sh
./scripts/deploy/start.sh
```

#### scripts/maintenance/
**维护脚本**：日常维护和优化

- `weekly_cleanup.sh` - 每周清理
- `security_check.sh` - 安全检查
- `prewarm-mcp.sh` - MCP 预热
- `database-optimization.sql` - 数据库优化

**使用方法**:
```bash
# 每周清理
./scripts/maintenance/weekly_cleanup.sh

# 安全检查
./scripts/maintenance/security_check.sh
```

---

### 3. `config/` - 配置文件

集中管理所有配置文件。

#### config/docker/
**Docker 配置**

- `docker-compose.yml` - 生产环境配置
- `docker-compose.dev.yml` - 开发环境配置

**使用方法**:
```bash
# 开发环境
docker-compose -f config/docker/docker-compose.dev.yml up

# 生产环境
docker-compose -f config/docker/docker-compose.yml up -d
```

#### config/
**其他配置**

- `turbo.json` - Turborepo 配置
- `mise.toml` - Mise 工具配置

---

### 4. `docs/` - 文档中心

**完整文档**: [docs/README.md](docs/README.md)

```
docs/
├── README.md                   # 文档导航中心
├── architecture/               # 架构文档 (1份)
├── security/                   # 安全文档 (2份)
├── optimization/               # 优化文档 (6份)
├── deployment/                 # 部署文档 (3份)
└── guides/                     # 使用指南 (5份)
```

**快速链接**:
- [完整文档导航](docs/README.md)
- [系统架构](docs/architecture/ARCHITECTURE.md)
- [快速开始](apps/rust-trading-bot/docs/user-guide/QUICKSTART.md)

---

### 5. `.archive/` - 归档目录

**已归档的历史文件** (Git 忽略)

```
.archive/
├── scripts/                    # 历史脚本
│   ├── detailed_protobuf_analysis.py
│   ├── find_matching_secret.py
│   └── ...其他历史脚本
│
├── venv/                       # Python 虚拟环境
├── .codex/                     # Codex 开发工具缓存
└── README.md                   # tools 历史说明
```

**说明**: 
- 此目录不会被提交到 Git
- 用于保存历史文件和临时开发工具
- 可以随时清空

---

## 🔑 关键文件说明

### 根目录文件

| 文件 | 说明 | 用途 |
|------|------|------|
| **README.md** | 项目主页 | 项目介绍、快速开始 |
| **PROJECT_STRUCTURE.md** | 本文件 | 项目结构说明 |
| **.env** | 环境变量 | 敏感配置 (gitignored) |
| **.env.example** | 环境变量模板 | 配置示例 |
| **package.json** | Node.js 配置 | 依赖和脚本 |
| **.gitignore** | Git 忽略 | 排除规则 |

---

## 📊 项目规模

### 代码统计

```
总代码行数: ~20,000 行
├── Rust:        ~15,000 行 (75%)
├── TypeScript:  ~3,000 行  (15%)
├── Python:      ~2,000 行  (10%)
```

### 文档统计

```
总文档数: 36 份
├── 项目文档:    19 份 (53%)
├── Rust Bot:    15 份 (42%)
├── 其他:        2 份  (5%)
```

### 目录大小

```
总大小: ~2.5 GB
├── node_modules:  ~1.8 GB  (72%)
├── target:        ~500 MB  (20%)
├── 源代码:        ~50 MB   (2%)
├── 文档:          ~2 MB    (0.1%)
├── 其他:          ~148 MB  (6%)
```

---

## 🎯 使用场景

### 场景 1: 运行交易机器人

```bash
# 1. 进入项目目录
cd /home/hanins/code/web3

# 2. 编译 Rust 项目
cd apps/rust-trading-bot
cargo build --release

# 3. 运行
./target/release/show_assets
```

### 场景 2: 启动监控服务

```bash
# 启动所有监控
node scripts/monitors/start_all_monitors.js

# 或单独启动
node scripts/monitors/start_6551_monitor.js
```

### 场景 3: 查看文档

```bash
# 文档中心
cat docs/README.md

# Rust Bot 文档
cat apps/rust-trading-bot/docs/README.md

# DeepSeek 文档
cat apps/rust-trading-bot/docs/deepseek/README.md
```

### 场景 4: 维护任务

```bash
# 每周清理
./scripts/maintenance/weekly_cleanup.sh

# 安全检查
./scripts/maintenance/security_check.sh
```

---

## 🔧 开发规范

### 文件命名

1. **文档**: 大写 + 下划线 (`SYSTEM_ARCHITECTURE.md`)
2. **脚本**: 小写 + 下划线 (`weekly_cleanup.sh`)
3. **代码**: 小写 + 下划线 (Rust/Python) 或驼峰 (TypeScript)

### 目录组织

1. **应用程序** → `apps/`
2. **脚本** → `scripts/`
3. **配置** → `config/`
4. **文档** → `docs/`
5. **归档** → `.archive/`

### 文档维护

1. 每个主要目录都有 `README.md`
2. 文档按功能分类
3. 保持链接有效
4. 定期更新

---

## 📝 变更日志

### v2.0 (2025-10-26)

**重大重构**:
- ✅ 整理根目录文件结构
- ✅ 创建 `scripts/` 统一脚本目录
- ✅ 创建 `config/` 统一配置目录
- ✅ 归档历史文件到 `.archive/`
- ✅ 完善文档体系
- ✅ 删除空目录 (`packages/`, `tools/`)

**文件移动**:
- ✅ 启动脚本 → `scripts/monitors/`
- ✅ 部署脚本 → `scripts/deploy/`
- ✅ 维护脚本 → `scripts/maintenance/`
- ✅ Docker 配置 → `config/docker/`
- ✅ 历史文件 → `.archive/`

### v1.0 (2025-10-20)

初始版本，文件分散。

---

## 💡 常见问题

### Q1: 如何查找某个功能的代码？

按功能查找：
- 交易功能 → `apps/rust-trading-bot/src/`
- 监控功能 → `apps/social-monitor/`
- AI 交易 → `apps/rust-trading-bot/src/deepseek_client.rs`

### Q2: 如何添加新脚本？

1. 确定脚本类型（监控/部署/维护）
2. 放到相应的 `scripts/` 子目录
3. 添加执行权限: `chmod +x`
4. 更新 `scripts/README.md`

### Q3: 如何查看文档？

从文档中心开始: `docs/README.md`

### Q4: `.archive/` 可以删除吗？

可以。它包含历史文件和临时工具，不影响项目运行。

---

## 🔗 相关链接

- [项目主页](README.md)
- [文档中心](docs/README.md)
- [Rust Trading Bot](apps/rust-trading-bot/docs/README.md)
- [DeepSeek AI](apps/rust-trading-bot/docs/deepseek/README.md)

---

**📁 项目结构清晰，易于维护！**

_最后更新: 2025-10-26_
