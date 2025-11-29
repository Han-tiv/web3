# 内存优化快速参考卡

## 🚀 快速修复步骤

### 1️⃣ 立即生效的修复
```bash
# 所有修复已应用到代码中
cd /home/hanins/code/web3/apps/rust-trading-bot
cargo build --release --bin integrated_ai_trader
```

### 2️⃣ 添加 Swap (强烈推荐)
```bash
sudo fallocate -l 4G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab
```

### 3️⃣ 使用智能编译脚本
```bash
# 检查资源后再编译
./scripts/smart_build.sh integrated_ai_trader
```

---

## 🔧 修复内容一览

| 问题 | 修复方案 | 位置 |
|------|---------|------|
| `tracked_coins` 无限增长 | 最多 100 个 + 24h 过期 | `src/bin/integrated_ai_trader.rs:174-175` |
| 持仓追踪器未清理 | 每 1 小时自动清理孤立追踪器 | `src/bin/integrated_ai_trader.rs:485` |
| API 调用卡死 | 10s/30s 超时保护 | `src/bin/integrated_ai_trader.rs:562,608` |
| 编译与运行冲突 | 智能编译脚本 + 资源检测 | `scripts/smart_build.sh` |
| 定期清理缺失 | 每 1 小时全局清理 | `src/bin/integrated_ai_trader.rs:383-390` |

---

## 📊 关键参数

```rust
max_tracked_coins: 100   // 最大追踪币种数
coin_ttl_hours: 24       // 币种过期时间 (小时)
SignalHistory::new(30)   // 信号历史上限

// API 超时设置
K线获取: 10秒
AI 分析: 30秒
```

---

## ✅ 验证清单

- [ ] 代码已编译通过 (`cargo check`)
- [ ] 添加了 Swap 分区
- [ ] 查看日志确认清理机制工作 (`grep "清理" logs/*.log`)
- [ ] 监控内存使用 (`free -h`)
- [ ] 不再出现 OOM 事件 (`journalctl | grep "killed"`)

---

## 🆘 应急命令

```bash
# 查看内存使用
free -h

# 杀死交易程序
pkill -f integrated_ai_trader

# 临时添加 2GB swap
sudo fallocate -l 2G /swapfile && sudo chmod 600 /swapfile && \
sudo mkswap /swapfile && sudo swapon /swapfile

# 重启程序
cargo run --release --bin integrated_ai_trader
```

---

## 📖 完整文档
详见: `docs/MEMORY_OPTIMIZATION.md`
