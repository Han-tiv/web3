# 🧹 Web3 项目安全清理指南

**策略**: 保留所有项目代码，只删除可重新生成的文件

---

## ✅ 保留策略

### 完全保留
- ✅ **所有项目源代码** (valuescan, social-monitor, nofx等)
- ✅ **所有配置文件** (.env, config/*)
- ✅ **所有数据库文件** (data/*.db)
- ✅ **所有文档** (归档但不删除)
- ✅ **已编译的二进制** (先备份)

### 可以删除 (可重新生成)
- 🗑️ **node_modules** (通过 pnpm install 恢复)
- 🗑️ **Rust 编译缓存** (target/，通过 cargo build 恢复)
- 🗑️ **Python 缓存** (__pycache__, *.pyc)
- 🗑️ **临时文件** (.pid, .swp, nohup.out等)
- 🗑️ **大型日志文件** (超过10MB的日志)

---

## 🚀 使用方法

### 方式1: 自动清理 (推荐)

```bash
cd /home/hanins/code/web3
./safe_cleanup.sh
```

**脚本会自动**:
1. 备份已编译的二进制文件
2. 删除所有 node_modules
3. 清理 Rust 编译缓存
4. 清理 Python 缓存
5. 清理临时文件
6. 压缩归档旧日志
7. 整理文档到 archive/

**预计耗时**: 2-5分钟  
**预计节省**: 500MB - 1.5GB

---

### 方式2: 手动清理

如果想要更精细的控制：

```bash
cd /home/hanins/code/web3

# 1. 备份二进制文件
mkdir -p binaries
cp apps/rust-trading-bot/target/release/integrated_ai_trader \
   binaries/integrated_ai_trader_$(date +%Y%m%d)

# 2. 删除 node_modules
find . -type d -name "node_modules" -not -path "*/venv/*" -exec rm -rf {} + 2>/dev/null

# 3. 清理 Rust 缓存
cd apps/rust-trading-bot
cargo clean
cd ../..

# 4. 清理 Python 缓存
find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null
find . -type f -name "*.pyc" -delete

# 5. 清理临时文件
rm -f monitor.pid
find . -name "*.swp" -delete
find . -name "nohup.out" -delete

# 6. 整理文档
cd apps/rust-trading-bot
mkdir -p archive/dev-logs-2025
mv B1_*.md archive/dev-logs-2025/ 2>/dev/null || true
mv PHASE_*.md archive/dev-logs-2025/ 2>/dev/null || true
```

---

## 📊 清理效果预估

| 项目 | 清理前 | 清理后 | 节省 |
|------|--------|--------|------|
| node_modules | 533MB | 0MB | 533MB |
| rust target/ | 500MB-1GB | 0MB | 500MB-1GB |
| Python缓存 | 10-50MB | 0MB | 10-50MB |
| 临时文件 | 5-10MB | 0MB | 5-10MB |
| **总计** | **~1.5GB** | **0MB** | **~1.5GB** |

**注意**: 所有被删除的内容都可以重新生成

---

## 🔄 恢复方法

### 恢复 node_modules

```bash
cd /home/hanins/code/web3
pnpm install
```

**耗时**: 2-5分钟  
**网络**: 需要联网下载依赖

---

### 恢复 Rust 编译缓存

```bash
cd /home/hanins/code/web3/apps/rust-trading-bot

# 开发模式 (快速编译)
cargo build

# 生产模式 (优化编译)
cargo build --release
```

**耗时**: 
- 开发模式: 5-10分钟
- 生产模式: 10-20分钟

**或者直接使用备份的二进制**:
```bash
cp binaries/integrated_ai_trader_20251129 target/release/integrated_ai_trader
chmod +x target/release/integrated_ai_trader
```

---

### 恢复 Python 虚拟环境 (如果删除了)

```bash
cd /home/hanins/code/web3/apps/python-telegram-monitor

# 创建虚拟环境
python3 -m venv venv

# 激活并安装依赖
source venv/bin/activate
pip install -r requirements.txt
```

---

## 📁 归档位置

清理脚本会将临时文档归档到以下位置（不删除）：

```
apps/rust-trading-bot/archive/
├── dev-logs-2025/          # 开发过程记录
│   ├── B1_*.md
│   ├── PHASE_*.md
│   ├── CRITICAL_ISSUES_SUMMARY.md
│   └── ...
├── mermaid-setup/          # Mermaid相关文档
│   └── *MERMAID*.md
└── feature-comparisons/    # 功能对比文档
    └── FULL_FEATURE_COMPARISON.md

apps/rust-trading-bot/logs/archive/
└── *.log.gz                # 压缩的旧日志

binaries/
└── integrated_ai_trader_*  # 二进制备份
```

---

## ⚠️ 注意事项

### 清理前检查

```bash
# 1. 确认rust-trading-bot正在运行
ps aux | grep integrated_ai_trader

# 2. 如果正在运行，先停止
pkill integrated_ai_trader

# 3. 执行清理
./safe_cleanup.sh

# 4. 重新启动
./target/release/integrated_ai_trader
# 或使用备份的二进制
./binaries/integrated_ai_trader_20251129
```

### 清理后验证

```bash
# 1. 检查项目结构
ls -la apps/

# 2. 确认源代码完整
ls apps/rust-trading-bot/src/
ls apps/python-telegram-monitor/

# 3. 确认配置文件存在
cat .env | head -5

# 4. 测试重新编译
cd apps/rust-trading-bot
cargo build --release
```

---

## 🎯 项目状态确认

所有项目都会被保留：

### ✅ rust-trading-bot (保留)
- 状态: 活跃，正在运行
- 用途: 主交易引擎
- 操作: 清理编译缓存，归档文档

### ✅ python-telegram-monitor (保留)
- 状态: 活跃
- 用途: Telegram信号监控
- 操作: 清理Python缓存，删除大日志

### ✅ valuescan (保留)
- 状态: 未运行，但保留代码
- 用途: 资金监控 (可能的备用系统)
- 操作: 只清理缓存，代码完整保留

### ✅ social-monitor (保留)
- 状态: 未运行，但保留代码
- 用途: 社交媒体监控
- 操作: 清理node_modules，代码保留

### ✅ nofx (保留)
- 状态: 未运行，但保留代码
- 用途: 待确认
- 操作: 清理node_modules，代码保留

### ✅ contract-auditor (保留)
- 状态: 工具项目
- 用途: 智能合约审计
- 操作: 清理缓存，代码保留

---

## 📝 后续步骤

### 清理后立即执行

```bash
# 1. 恢复 node_modules (如果需要运行某些项目)
cd /home/hanins/code/web3
pnpm install

# 2. 重新编译 rust-trading-bot (如果需要重新启动)
cd apps/rust-trading-bot
cargo build --release

# 3. 启动交易系统
./target/release/integrated_ai_trader

# 或使用备份的二进制
cd /home/hanins/code/web3
./binaries/integrated_ai_trader_20251129
```

### 定期维护

```bash
# 每周执行清理
cd /home/hanins/code/web3
./safe_cleanup.sh

# 每月检查
- 查看归档文档是否需要删除
- 清理旧的二进制备份 (保留最近3个)
- 检查日志归档大小
```

---

## 🔧 自定义清理

如果需要调整清理策略，编辑 `safe_cleanup.sh`：

```bash
# 编辑脚本
nano safe_cleanup.sh

# 常见自定义:
# - 修改日志保留天数 (默认7天)
# - 修改日志大小阈值 (默认10MB)
# - 添加/移除要清理的文件类型
# - 调整归档目录结构
```

---

<div align="center">

# ✅ 安全第一

**此清理方案**:
- ✅ 保留所有项目源代码
- ✅ 保留所有配置和数据
- ✅ 只删除可重新生成的文件
- ✅ 备份关键二进制文件
- ✅ 归档而不是删除文档

**任何时候都可以完全恢复**

---

**准备好了？**

```bash
cd /home/hanins/code/web3
./safe_cleanup.sh
```

</div>
