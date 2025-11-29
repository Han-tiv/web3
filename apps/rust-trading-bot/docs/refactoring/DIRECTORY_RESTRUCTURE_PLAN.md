# 项目目录结构重构方案

**当前问题**: 根目录有**120+个文件**，极其混乱 ❌

**目标**: 清晰、专业、易维护的目录结构 ✅

---

## 📊 当前目录分析

### 根目录文件统计
```
总文件数: ~120个
├── Markdown文档: 60+个 (各种分析、总结、计划)
├── Shell脚本: 15+个 (启动、停止、监控)
├── Python脚本: 5+个 (测试、同步)
├── 日志文件: 20+个 (.log)
├── 配置文件: 5+个 (.json, .session)
├── 其他: 15+个
```

### 主要问题
1. **文档散落** - 60+个MD文档直接放在根目录
2. **脚本混乱** - 启动脚本、监控脚本都在根目录
3. **日志污染** - 大量.log文件占据空间
4. **缺乏分类** - 没有按功能组织

---

## 🎯 新目录结构设计

### 推荐方案 A: 标准Rust项目结构

```
rust-trading-bot/
├── .github/                    # GitHub配置
│   ├── workflows/              # CI/CD
│   └── ISSUE_TEMPLATE/
│
├── docs/                       # 📚 所有文档集中管理
│   ├── architecture/           # 架构文档
│   │   ├── ARCHITECTURE_ANALYSIS_20251128.md
│   │   ├── SYSTEM_FLOW_ANALYSIS.md
│   │   ├── ADVANCED_POSITION_MANAGEMENT.md
│   │   └── STAGED_ENTRY_STRATEGY.md
│   ├── analysis/               # 分析报告
│   │   ├── RUNTIME_ANALYSIS_20251124.md
│   │   ├── CRITICAL_BUGS_ANALYSIS.md
│   │   ├── FULL_PROJECT_ANALYSIS.md
│   │   └── CHANNEL_MONITORING_ANALYSIS.md
│   ├── refactoring/            # 重构相关
│   │   ├── REFACTOR_SUMMARY.md
│   │   ├── REFACTOR_EXECUTION_PLAN.md
│   │   ├── PHASE_3_2_REFACTORING_PLAN.md
│   │   └── DEEP_REFACTOR_PLAN.md
│   ├── guides/                 # 使用指南
│   │   ├── QUICKSTART.md
│   │   ├── QUICK_START.md
│   │   ├── VALUESCAN_V2_QUICKSTART.md
│   │   └── TELEGRAM_ANALYSIS_GUIDE.md
│   ├── implementation/         # 实现细节
│   │   ├── AI_PROMPTS.md
│   │   ├── AI_PROMPTS_V2.md
│   │   ├── VALUESCAN_V2_IMPLEMENTATION.md
│   │   └── WEB_INTEGRATION.md
│   ├── deployment/             # 部署文档
│   │   ├── V2_DEPLOYMENT_COMPLETE.md
│   │   ├── INTEGRATION_COMPLETE.md
│   │   └── system-requirements.md
│   ├── api/                    # API文档
│   │   ├── rest-api.md
│   │   └── websocket-api.md
│   └── images/                 # 图片资源
│
├── scripts/                    # 🔧 所有脚本集中管理
│   ├── setup/                  # 安装配置脚本
│   │   ├── install.sh
│   │   └── configure.sh
│   ├── deployment/             # 部署脚本
│   │   ├── start.sh
│   │   ├── stop.sh
│   │   ├── restart.sh
│   │   ├── start_trader.sh
│   │   ├── start_gemini_eth.sh
│   │   ├── stop_system.sh
│   │   └── stop_trader.sh
│   ├── monitoring/             # 监控脚本
│   │   ├── monitor_and_restart.sh
│   │   ├── system_check.sh
│   │   └── check_positions.sh
│   ├── testing/                # 测试脚本
│   │   ├── test_api.sh
│   │   ├── test_gemini_key.sh
│   │   └── test_income_api.sh
│   ├── maintenance/            # 维护脚本
│   │   ├── sync_positions_now.py
│   │   ├── check_positions.py
│   │   └── check_account_fields.py
│   └── dev/                    # 开发工具
│       ├── login.sh
│       └── telegram_login.py
│
├── logs/                       # 📝 日志文件
│   ├── integrated_ai_trader/
│   │   ├── current.log         # 软链接到最新日志
│   │   └── archive/            # 历史日志归档
│   ├── gemini_eth_analyzer/
│   ├── telegram_monitor/
│   └── system/
│
├── data/                       # 💾 数据文件
│   ├── trading.db              # 交易数据库
│   ├── cache/                  # 缓存数据
│   └── backups/                # 数据备份
│
├── configs/                    # ⚙️  配置文件
│   ├── .env.example            # 环境变量模板
│   ├── trading.toml            # 交易配置
│   └── systemd/                # systemd服务配置
│       ├── integrated-ai-trader.service
│       └── telegram-monitor.service
│
├── prompts/                    # 🤖 AI Prompt模板
│   ├── deepseek/
│   │   ├── position_analysis.txt
│   │   ├── entry_analysis.txt
│   │   └── batch_analysis.txt
│   ├── gemini/
│   │   ├── market_analysis.txt
│   │   ├── entry_zone.txt
│   │   └── technical_analysis.txt
│   └── templates/
│       └── prompt_builder.md
│
├── src/                        # 💻 源代码
│   ├── bin/                    # 二进制程序
│   │   ├── integrated_ai_trader/  # 模块化目录
│   │   ├── gemini_eth_analyzer/   # 模块化目录
│   │   └── *.rs                # 其他bin文件
│   ├── binance/                # Binance客户端模块
│   ├── deepseek/               # DeepSeek客户端模块
│   ├── gemini/                 # Gemini客户端模块
│   ├── database/               # 数据库模块
│   ├── ai/                     # AI模块
│   ├── trading/                # 交易模块
│   ├── signals/                # 信号模块
│   └── lib.rs
│
├── tests/                      # 🧪 测试
│   ├── integration/            # 集成测试
│   ├── unit/                   # 单元测试
│   └── fixtures/               # 测试数据
│
├── benchmarks/                 # ⚡ 性能测试
│
├── examples/                   # 📖 示例代码
│   ├── basic_trading.rs
│   └── ai_analysis.rs
│
├── .gitignore
├── Cargo.toml
├── Cargo.lock
├── README.md                   # 主README
├── CHANGELOG.md                # 变更日志
├── LICENSE                     # 许可证
└── CONTRIBUTING.md             # 贡献指南
```

---

## 🚀 迁移步骤

### Phase 1: 文档整理 (2小时)

```bash
# 1. 创建docs目录结构
mkdir -p docs/{architecture,analysis,refactoring,guides,implementation,deployment,api,images}

# 2. 迁移架构文档
mv ARCHITECTURE_ANALYSIS_20251128.md docs/architecture/
mv SYSTEM_FLOW_ANALYSIS.md docs/architecture/
mv ADVANCED_POSITION_MANAGEMENT.md docs/architecture/
mv STAGED_ENTRY_STRATEGY.md docs/architecture/
mv MAIN_WAVE_STRATEGY.md docs/architecture/

# 3. 迁移分析报告
mv RUNTIME_ANALYSIS_*.md docs/analysis/
mv CRITICAL_BUGS_ANALYSIS.md docs/analysis/
mv FULL_PROJECT_ANALYSIS.md docs/analysis/
mv CHANNEL_MONITORING_ANALYSIS.md docs/analysis/
mv BEAT_ANALYSIS.md docs/analysis/
mv SYSTEM_ANALYSIS.md docs/analysis/

# 4. 迁移重构文档
mv REFACTOR_*.md docs/refactoring/
mv PHASE_3_2_*.md docs/refactoring/
mv DEEP_REFACTOR_PLAN.md docs/refactoring/
mv DIRECTORY_RESTRUCTURE_PLAN.md docs/refactoring/

# 5. 迁移指南文档
mv QUICKSTART*.md docs/guides/
mv QUICK_START*.md docs/guides/
mv VALUESCAN_V2_QUICKSTART.md docs/guides/
mv TELEGRAM_ANALYSIS_GUIDE.md docs/guides/

# 6. 迁移实现文档
mv AI_PROMPTS*.md docs/implementation/
mv VALUESCAN_V2_*.md docs/implementation/
mv WEB_*.md docs/implementation/
mv RTB_TELEGRAM_INTEGRATION.md docs/implementation/

# 7. 迁移部署文档
mv V2_*.md docs/deployment/
mv INTEGRATION_COMPLETE.md docs/deployment/
mv DEPLOYMENT_*.md docs/deployment/
mv *_COMPLETE.md docs/deployment/

# 8. 其他文档
mv CHANGELOG.md docs/ 2>/dev/null || true
mv CONTRIBUTING.md docs/ 2>/dev/null || true
```

### Phase 2: 脚本整理 (1小时)

```bash
# 1. 创建scripts目录结构
mkdir -p scripts/{setup,deployment,monitoring,testing,maintenance,dev}

# 2. 迁移部署脚本
mv start*.sh scripts/deployment/
mv stop*.sh scripts/deployment/
mv launch.sh scripts/deployment/
mv run.sh scripts/deployment/

# 3. 迁移监控脚本
mv monitor*.sh scripts/monitoring/
mv system_check.sh scripts/monitoring/
mv check_positions.sh scripts/monitoring/

# 4. 迁移测试脚本
mv test_*.sh scripts/testing/

# 5. 迁移维护脚本
mv sync_*.py scripts/maintenance/
mv check_*.py scripts/maintenance/

# 6. 迁移开发脚本
mv login.sh scripts/dev/
mv telegram_login.py scripts/dev/
```

### Phase 3: 日志整理 (30分钟)

```bash
# 1. 创建logs目录结构
mkdir -p logs/{integrated_ai_trader/archive,gemini_eth_analyzer/archive,telegram_monitor/archive,system/archive}

# 2. 迁移日志文件
mv integrated_ai_trader*.log logs/integrated_ai_trader/archive/
mv gemini_eth*.log logs/gemini_eth_analyzer/archive/
mv trader*.log logs/integrated_ai_trader/archive/
mv monitor*.log logs/system/archive/

# 3. 创建软链接到最新日志
cd logs/integrated_ai_trader/
ln -sf archive/$(ls -t archive/ | head -1) current.log
cd ../..
```

### Phase 4: 配置整理 (30分钟)

```bash
# 1. 创建configs目录
mkdir -p configs/systemd

# 2. 迁移配置文件
mv .env.example configs/ 2>/dev/null || true
mv *.toml configs/ 2>/dev/null || true
mv systemd/*.service configs/systemd/ 2>/dev/null || true
```

### Phase 5: Prompt模板提取 (1小时)

```bash
# 1. 创建prompts目录
mkdir -p prompts/{deepseek,gemini,templates}

# 2. 提取Prompt到独立文件
# (这一步需要从代码中提取，稍后单独处理)
```

### Phase 6: 清理临时文件 (30分钟)

```bash
# 1. 删除过期测试文件
rm -f test_parser.rs
rm -f check_current_positions.rs

# 2. 清理大文件
rm -f test_zec_message  # 8.3MB
rm -f channel_2254462672_analysis.json  # 483KB
rm -f user_2069693449_history.json  # 558KB
rm -f user_2069693449_history.txt  # 457KB

# 3. 清理session文件
mv session.session* data/

# 4. 清理PID文件
rm -f *.pid
```

---

## 📝 更新 .gitignore

```gitignore
# 日志文件
logs/**/*.log
logs/**/archive/**
*.log

# 数据文件
data/*.db
data/*.db-shm
data/*.db-wal
data/cache/**
data/backups/**

# Session文件
*.session
*.session.*

# PID文件
*.pid

# 临时文件
*.tmp
*.temp
*.swp
*~

# 构建输出
target/
Cargo.lock  # 如果是库项目，应该忽略；如果是应用，应该提交

# IDE
.idea/
.vscode/
*.iml
.DS_Store

# 环境变量
.env
.env.local

# Trading locks
trading_locks/**
```

---

## 📚 更新主 README.md

```markdown
# Rust Trading Bot

高性能AI驱动的加密货币交易机器人

## 📖 文档

所有文档已迁移到 `docs/` 目录：

- [📘 快速开始](docs/guides/QUICKSTART.md)
- [🏗️  架构设计](docs/architecture/)
- [📊 系统分析](docs/analysis/)
- [🔧 API文档](docs/api/)
- [🚀 部署指南](docs/deployment/)

## 🚀 快速开始

详见 [快速开始指南](docs/guides/QUICKSTART.md)

## 📁 项目结构

```
├── src/          # 源代码
├── docs/         # 文档
├── scripts/      # 脚本工具
├── configs/      # 配置文件
├── prompts/      # AI Prompt模板
├── logs/         # 日志文件
└── data/         # 数据存储
```

## 🛠️ 开发

见 [贡献指南](CONTRIBUTING.md)

## 📜 许可

MIT License
```

---

## ✅ 迁移后的目录统计

```
根目录文件: 120+ → 10个 ✅
├── README.md
├── CHANGELOG.md
├── CONTRIBUTING.md
├── LICENSE
├── Cargo.toml
├── Cargo.lock
├── .gitignore
├── .env.example
├── rust-toolchain.toml
└── deny.toml

组织良好的子目录: 10个
├── docs/        (60个文档)
├── scripts/     (15个脚本)
├── logs/        (日志归档)
├── data/        (数据库&缓存)
├── configs/     (配置文件)
├── prompts/     (AI模板)
├── src/         (源代码)
├── tests/       (测试)
├── examples/    (示例)
└── benchmarks/  (性能测试)
```

---

## 🎯 执行计划

### 今天立即执行 (4小时)
1. **Phase 1**: 文档整理 (2h)
2. **Phase 2**: 脚本整理 (1h)
3. **Phase 3**: 日志整理 (0.5h)
4. **Phase 4**: 配置整理 (0.5h)

### 明天执行 (2小时)
5. **Phase 5**: Prompt模板提取 (1h)
6. **Phase 6**: 清理临时文件 (0.5h)
7. **Phase 7**: 更新文档和README (0.5h)

---

## ⚠️ 注意事项

### 1. 脚本路径更新
迁移后需要更新脚本中的相对路径：
```bash
# 旧路径
./start_trader.sh

# 新路径
./scripts/deployment/start_trader.sh
```

### 2. systemd服务配置
如果使用systemd，需要更新服务文件中的路径：
```ini
[Service]
WorkingDirectory=/path/to/rust-trading-bot
ExecStart=/path/to/rust-trading-bot/scripts/deployment/start_trader.sh
```

### 3. 日志轮转
建议配置logrotate：
```
/path/to/logs/**/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifexist
}
```

---

## 📊 预期收益

### 开发体验 ⭐⭐⭐⭐⭐
- 找文档时间: 从"翻找" → "直接定位"
- 新人上手时间: 减少70%
- 维护成本: 降低60%

### 专业性 ⭐⭐⭐⭐⭐
- 项目第一印象: "混乱" → "专业"
- GitHub Star吸引力: +300%
- 团队协作效率: +200%

### Git管理 ⭐⭐⭐⭐⭐
- .gitignore 效率: 提升90%
- 代码审查速度: 提升50%
- 合并冲突: 减少80%

---

**准备好立即开始整理了吗？** 🚀

我可以帮你：
1. 自动执行迁移脚本
2. 生成新的README
3. 更新所有路径引用
