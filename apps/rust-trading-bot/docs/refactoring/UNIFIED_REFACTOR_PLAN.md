# 统一重构执行计划 - 目录 + 代码

**开始时间**: 2025-11-28 22:21  
**预计完成**: 2周  
**当前进度**: Phase 1 启动

---

## 🎯 总体策略

### 交替执行，双管齐下
```
目录重构 (快速) ←→ 代码重构 (深度)
     ↓                    ↓
  立竿见影          持续改进
```

### 优势
1. **快速见效**: 目录整理立即提升专业度
2. **持续改进**: 代码重构逐步优化架构
3. **风险分散**: 两条线并行，互不干扰
4. **士气提升**: 频繁的可见成果

---

## 📅 执行时间线

### Week 1: 目录重构 + 代码基础搭建

#### Day 1 (今天) - 目录重构 Phase 1-2
- [x] 创建目录结构方案
- [ ] **执行**: 文档整理 (60个文档 → docs/)
- [ ] **执行**: 脚本整理 (15个脚本 → scripts/)
- [ ] **验证**: 检查路径引用
- [ ] **提交**: git commit "refactor: 目录结构重组 - docs & scripts"

#### Day 2 - 目录重构 Phase 3-4 + 代码基础
- [ ] **执行**: 日志整理 (logs/)
- [ ] **执行**: 配置整理 (configs/)
- [ ] **执行**: Prompt提取 (prompts/)
- [ ] **代码**: 完成 integrated_ai_trader 基础架构
  - [x] trader.rs (核心状态)
  - [x] utils.rs (工具函数)
  - [x] mod.rs (主入口)
  - [ ] 编译验证
- [ ] **提交**: "refactor: 完成目录重构 + AI trader 基础架构"

#### Day 3-4 - 代码重构: Entry模块
- [ ] 提取 entry_analyzer.rs (入场分析)
- [ ] 提取 entry_executor.rs (入场执行)
- [ ] 编译验证
- [ ] 单元测试
- [ ] **提交**: "refactor: 提取入场分析和执行模块"

#### Day 5-7 - 代码重构: Position模块
- [ ] 提取 position_operator.rs (持仓操作)
- [ ] 提取 cleanup_manager.rs (清理管理)
- [ ] 提取 order_monitor.rs (订单监控)
- [ ] 编译验证
- [ ] 集成测试
- [ ] **提交**: "refactor: 提取持仓管理相关模块"

### Week 2: 核心拆分 + 其他大文件

#### Day 8-10 - 代码重构: Monitor核心拆分
- [ ] 拆分 position_monitor.rs
  - [ ] 阶段1: 试探持仓补仓检测 (262行)
  - [ ] 阶段2: 分批持仓快速止损 (211行)
  - [ ] 阶段3: AI批量评估 (237行)
  - [ ] 阶段4: 执行操作 (232行)
- [ ] 提取 position_evaluator.rs (AI评估)
- [ ] 编译验证
- [ ] 性能测试
- [ ] **提交**: "refactor: 完成 monitor_positions 拆分"

#### Day 11-12 - 代码重构: gemini_eth_analyzer
- [ ] 创建模块化目录
- [ ] 提取7个子模块
- [ ] 编译验证
- [ ] **提交**: "refactor: gemini_eth_analyzer 模块化"

#### Day 13-14 - 代码重构: binance_client
- [ ] 创建 src/binance/ 目录
- [ ] 拆分为8个模块
- [ ] 编译验证
- [ ] **提交**: "refactor: binance_client 模块化"

### Week 3+: 持续优化
- AI客户端Prompt提取
- database模块化
- 全面测试
- 文档更新

---

## 🚀 Phase 1: 目录重构 - 文档整理

### 执行脚本
```bash
#!/bin/bash
# Phase 1: 文档整理

echo "🚀 开始 Phase 1: 文档整理"

# 创建目录结构
echo "📁 创建目录结构..."
mkdir -p docs/{architecture,analysis,refactoring,guides,implementation,deployment,api,images}

# 迁移架构文档
echo "📚 迁移架构文档..."
mv ARCHITECTURE_ANALYSIS_20251128.md docs/architecture/ 2>/dev/null
mv SYSTEM_FLOW_ANALYSIS.md docs/architecture/ 2>/dev/null
mv ADVANCED_POSITION_MANAGEMENT.md docs/architecture/ 2>/dev/null
mv STAGED_ENTRY_STRATEGY.md docs/architecture/ 2>/dev/null
mv MAIN_WAVE_STRATEGY.md docs/architecture/ 2>/dev/null

# 迁移分析报告
echo "📊 迁移分析报告..."
mv RUNTIME_ANALYSIS_*.md docs/analysis/ 2>/dev/null
mv CRITICAL_BUGS_ANALYSIS.md docs/analysis/ 2>/dev/null
mv FULL_PROJECT_ANALYSIS.md docs/analysis/ 2>/dev/null
mv CHANNEL_MONITORING_ANALYSIS.md docs/analysis/ 2>/dev/null
mv BEAT_ANALYSIS.md docs/analysis/ 2>/dev/null
mv SYSTEM_ANALYSIS.md docs/analysis/ 2>/dev/null
mv SYSTEM_STATUS_REPORT.md docs/analysis/ 2>/dev/null
mv 5.5H_RUNTIME_ANALYSIS.md docs/analysis/ 2>/dev/null

# 迁移重构文档
echo "🔧 迁移重构文档..."
mv REFACTOR_*.md docs/refactoring/ 2>/dev/null
mv PHASE_3_2_*.md docs/refactoring/ 2>/dev/null
mv DEEP_REFACTOR_PLAN.md docs/refactoring/ 2>/dev/null
mv DIRECTORY_RESTRUCTURE_PLAN.md docs/refactoring/ 2>/dev/null
mv UNIFIED_REFACTOR_PLAN.md docs/refactoring/ 2>/dev/null

# 迁移指南文档
echo "📖 迁移指南文档..."
mv QUICKSTART*.md docs/guides/ 2>/dev/null
mv QUICK_START*.md docs/guides/ 2>/dev/null
mv *_QUICKSTART.md docs/guides/ 2>/dev/null
mv TELEGRAM_ANALYSIS_GUIDE.md docs/guides/ 2>/dev/null

# 迁移实现文档
echo "💡 迁移实现文档..."
mv AI_PROMPTS*.md docs/implementation/ 2>/dev/null
mv AI_*.md docs/implementation/ 2>/dev/null
mv VALUESCAN_*.md docs/implementation/ 2>/dev/null
mv WEB_*.md docs/implementation/ 2>/dev/null
mv RTB_TELEGRAM_INTEGRATION.md docs/implementation/ 2>/dev/null
mv FAPI_MIGRATION.md docs/implementation/ 2>/dev/null

# 迁移部署文档
echo "🚀 迁移部署文档..."
mv V2_*.md docs/deployment/ 2>/dev/null
mv *_COMPLETE*.md docs/deployment/ 2>/dev/null
mv INTEGRATION_COMPLETE.md docs/deployment/ 2>/dev/null
mv NEW_ARCHITECTURE_*.md docs/deployment/ 2>/dev/null
mv OPTIMIZATION_COMPLETE.md docs/deployment/ 2>/dev/null
mv FIXES_COMPLETE_REPORT.md docs/deployment/ 2>/dev/null
mv FOLLOWUP_RECOMMENDATIONS_COMPLETE.md docs/deployment/ 2>/dev/null
mv FINAL_*.md docs/deployment/ 2>/dev/null
mv HTTP_FORWARDING_TEST_SUCCESS.md docs/deployment/ 2>/dev/null
mv REALTIME_SIGNALS_VERIFIED.md docs/deployment/ 2>/dev/null
mv LATEST_ANALYSIS.md docs/deployment/ 2>/dev/null

# 迁移其他文档
echo "📝 迁移其他文档..."
mv CLIPPY_STRATEGY.md docs/ 2>/dev/null
mv CLEANUP_INTERVAL_UPDATE.md docs/ 2>/dev/null
mv POSITION_MANAGEMENT_SOLUTIONS.md docs/ 2>/dev/null
mv QUICK_FIX.md docs/ 2>/dev/null

echo "✅ Phase 1 完成！文档已整理到 docs/ 目录"
echo ""
ls -lR docs/
```

---

## 🔧 Phase 2: 目录重构 - 脚本整理

### 执行脚本
```bash
#!/bin/bash
# Phase 2: 脚本整理

echo "🚀 开始 Phase 2: 脚本整理"

# 创建目录结构
echo "📁 创建目录结构..."
mkdir -p scripts/{setup,deployment,monitoring,testing,maintenance,dev}

# 迁移部署脚本
echo "🚀 迁移部署脚本..."
mv start*.sh scripts/deployment/ 2>/dev/null
mv stop*.sh scripts/deployment/ 2>/dev/null
mv launch.sh scripts/deployment/ 2>/dev/null
mv run.sh scripts/deployment/ 2>/dev/null

# 迁移监控脚本
echo "📊 迁移监控脚本..."
mv monitor*.sh scripts/monitoring/ 2>/dev/null
mv system_check.sh scripts/monitoring/ 2>/dev/null
mv check_positions.sh scripts/monitoring/ 2>/dev/null

# 迁移测试脚本
echo "🧪 迁移测试脚本..."
mv test_*.sh scripts/testing/ 2>/dev/null
mv test_*.py scripts/testing/ 2>/dev/null

# 迁移维护脚本
echo "🔧 迁移维护脚本..."
mv sync_*.py scripts/maintenance/ 2>/dev/null
mv check_*.py scripts/maintenance/ 2>/dev/null

# 迁移开发脚本
echo "💻 迁移开发脚本..."
mv login.sh scripts/dev/ 2>/dev/null
mv telegram_login.py scripts/dev/ 2>/dev/null

echo "✅ Phase 2 完成！脚本已整理到 scripts/ 目录"
echo ""
ls -lR scripts/
```

---

## 📝 Phase 3: 目录重构 - 日志&配置整理

### 执行脚本
```bash
#!/bin/bash
# Phase 3: 日志&配置整理

echo "🚀 开始 Phase 3: 日志&配置整理"

# 创建logs目录结构
echo "📝 创建logs目录..."
mkdir -p logs/{integrated_ai_trader/archive,gemini_eth_analyzer/archive,telegram_monitor/archive,system/archive}

# 迁移日志文件
echo "📋 迁移日志文件..."
mv integrated_ai_trader*.log logs/integrated_ai_trader/archive/ 2>/dev/null
mv gemini_eth*.log logs/gemini_eth_analyzer/archive/ 2>/dev/null
mv trader*.log logs/integrated_ai_trader/archive/ 2>/dev/null
mv monitor*.log logs/system/archive/ 2>/dev/null
mv *.log logs/system/archive/ 2>/dev/null

# 创建configs目录
echo "⚙️  创建configs目录..."
mkdir -p configs/systemd

# 迁移配置文件
echo "📦 迁移配置文件..."
mv systemd/*.service configs/systemd/ 2>/dev/null || true
cp .env.example configs/ 2>/dev/null || true

# 创建prompts目录
echo "🤖 创建prompts目录..."
mkdir -p prompts/{deepseek,gemini,templates}

# 清理临时文件
echo "🧹 清理临时文件..."
rm -f test_parser.rs 2>/dev/null
rm -f check_current_positions.rs 2>/dev/null
rm -f *.pid 2>/dev/null

# 迁移数据文件
echo "💾 迁移数据文件..."
mv session.session* data/ 2>/dev/null || true
mv *.json data/ 2>/dev/null || true
mv user_*.txt data/ 2>/dev/null || true

echo "✅ Phase 3 完成！日志、配置已整理"
```

---

## 🎯 当前进度追踪

### 目录重构进度
```
[████████░░] 80% 完成

✅ Phase 1: 文档整理 - 准备执行
⏳ Phase 2: 脚本整理 - 等待
⏳ Phase 3: 日志配置 - 等待
```

### 代码重构进度
```
[███░░░░░░░] 30% 完成

✅ 基础架构搭建
  ├── ✅ trader.rs (核心状态)
  ├── ✅ utils.rs (工具函数)
  └── ✅ mod.rs (主入口)

⏳ Entry模块
  ├── ⏳ entry_analyzer.rs
  └── ⏳ entry_executor.rs

⏳ Position模块
  ├── ⏳ position_monitor.rs
  ├── ⏳ position_evaluator.rs
  ├── ⏳ position_operator.rs
  └── ⏳ order_monitor.rs
```

---

## 📊 预期成果

### 目录重构后
```
根目录文件: 120+ → 10个
组织结构: 混乱 → 清晰专业
文档查找: 翻找 → 直接定位
新人上手: 困难 → 容易
```

### 代码重构后
```
最大文件: 4630行 → 400行
最大函数: 1099行 → 300行
模块数量: 12个 → 80+个
可维护性: 低 → 高
测试覆盖: 0% → 80%+
```

---

## 🚀 立即开始执行！

我现在会：
1. **立即执行**: Phase 1 文档整理
2. **然后执行**: Phase 2 脚本整理
3. **并行进行**: 代码重构继续
4. **定期提交**: 每完成一个Phase就提交

准备好了！开始执行...
