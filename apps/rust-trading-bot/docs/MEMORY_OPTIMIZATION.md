# 服务器资源优化完整方案

## 问题分析总结

### 🔴 根本原因
1. **内存泄漏**: `tracked_coins` HashMap 无限增长,从不清理
2. **编译冲突**: Rust 编译与程序运行同时进行,7.8GB 内存不足
3. **并发问题**: 多个线程持续运行,缺少超时和清理机制
4. **缺少 Swap**: 系统无交换分区,OOM 立即触发进程终止

---

## ✅ 已实施的修复

### 1. 内存泄漏修复
#### ✅ 添加自动清理机制
- **tracked_coins 限制**: 最多 100 个币种
- **过期时间**: 24 小时自动清理
- **定期清理**: 每 1 小时执行一次全局清理

```rust
// 新增配置
max_tracked_coins: 100,  // 最多追踪 100 个币种
coin_ttl_hours: 24,      // 24 小时后自动过期

// 新增方法
cleanup_tracked_coins()      // 清理过期币种
cleanup_orphaned_trackers()  // 清理孤立的持仓追踪器
```

#### ✅ 持仓追踪器清理
- **孤立追踪器检测**: 每 1 小时检查是否有无效持仓
- **异常清理**: 超过 24 小时无法验证的追踪器自动删除

### 2. 并发优化
#### ✅ API 调用超时保护
- **K线获取超时**: 10 秒
- **AI 分析超时**: 30 秒
- **避免卡死**: 超时自动放弃,不阻塞主线程

```rust
// 添加 timeout 包装
tokio::time::timeout(
    tokio::time::Duration::from_secs(10),
    self.exchange.get_klines(...)
).await
```

### 3. 编译优化
#### ✅ 智能编译脚本
创建 `scripts/smart_build.sh`:
- **检测运行程序**: 编译前检查是否有交易程序运行
- **资源检查**: 内存/CPU 使用率预警
- **并行限制**: 限制为 2 个并行任务 (减少内存)
- **增量编译**: 启用 `CARGO_INCREMENTAL=1`

使用方法:
```bash
# 编译单个程序
./scripts/smart_build.sh integrated_ai_trader

# 查看帮助
./scripts/smart_build.sh --help
```

---

## 🔧 系统层面优化建议

### 1. 添加 Swap 分区 (强烈建议)
**目的**: 防止 OOM Killer 杀死进程

```bash
# 创建 4GB swap 文件
sudo fallocate -l 4G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile

# 永久启用 (添加到 /etc/fstab)
echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab

# 调整 swappiness (建议值 10-20)
sudo sysctl vm.swappiness=10
echo 'vm.swappiness=10' | sudo tee -a /etc/sysctl.conf
```

### 2. 限制程序内存使用
使用 systemd 或 cgroups 限制:

```bash
# 限制为 2GB 内存
cargo run --bin integrated_ai_trader &
PID=$!
echo $PID > /sys/fs/cgroup/memory/trading-bot/cgroup.procs
echo 2G > /sys/fs/cgroup/memory/trading-bot/memory.limit_in_bytes
```

### 3. 编译与运行分离
**强烈建议**: 不要在程序运行时编译

```bash
# 方案A: 使用 smart_build.sh (已自动检测)
./scripts/smart_build.sh

# 方案B: 先停止程序,再编译
pkill -f integrated_ai_trader
cargo build --release --bin integrated_ai_trader
cargo run --release --bin integrated_ai_trader

# 方案C: 使用 nohup 后台运行,避免 SSH 断开
nohup cargo run --release --bin integrated_ai_trader > trader.log 2>&1 &
```

---

## 📊 监控和预警

### 1. 实时监控脚本
创建 `scripts/monitor_resources.sh`:

```bash
#!/bin/bash
while true; do
    clear
    echo "=== 系统资源监控 ==="
    echo ""
    echo "内存使用:"
    free -h
    echo ""
    echo "运行中的交易程序:"
    ps aux | grep -E "integrated_ai_trader|deepseek_trader" | grep -v grep
    echo ""
    echo "持仓追踪器/币种统计:"
    # 可以通过日志解析显示
    echo ""
    sleep 5
done
```

### 2. 日志分析
查看清理日志:
```bash
# 查看最近的清理记录
grep "清理" trader.log | tail -20

# 查看内存相关日志
grep "追踪币种数\|追踪器数" trader.log | tail -20
```

---

## 🎯 使用建议

### 编译流程
```bash
# 1. 停止运行中的程序
pkill -f integrated_ai_trader

# 2. 使用智能编译脚本
cd /home/hanins/code/web3/apps/rust-trading-bot
./scripts/smart_build.sh integrated_ai_trader release

# 3. 启动程序 (使用 nohup 后台运行)
nohup cargo run --release --bin integrated_ai_trader > logs/trader_$(date +%Y%m%d).log 2>&1 &

# 4. 查看实时日志
tail -f logs/trader_$(date +%Y%m%d).log
```

### 运维最佳实践
1. **定期重启**: 每周重启一次程序,清理潜在的内存碎片
2. **日志轮转**: 使用 `logrotate` 避免日志文件过大
3. **监控告警**: 设置内存/CPU 告警 (可用 Prometheus + Grafana)
4. **资源预留**: 为系统保留至少 1GB 内存

---

## 🔍 验证修复效果

### 1. 查看当前内存使用
```bash
free -h
ps aux | grep integrated_ai_trader | head -1
```

### 2. 监控清理日志
```bash
# 实时查看清理操作
tail -f logs/trader_*.log | grep -E "清理|追踪"
```

### 3. 检查是否还有 OOM 事件
```bash
# 查看系统日志
sudo journalctl -u init.scope --since "1 hour ago" | grep -i "killed"
```

---

## 📝 配置参数调优

如果内存依然紧张,可以调整以下参数:

```rust
// src/bin/integrated_ai_trader.rs (第 174-175 行)
max_tracked_coins: 50,   // 降低到 50 个币种
coin_ttl_hours: 12,      // 降低到 12 小时

// 信号历史大小 (第 179 行)
SignalHistory::new(20),  // 降低到 20 条
```

---

## 🆘 应急处理

如果程序再次被 OOM Killer 杀死:

```bash
# 1. 立即检查内存
free -h

# 2. 查找占用最大的进程
ps aux --sort=-%mem | head -10

# 3. 临时添加 swap (如果还没有)
sudo fallocate -l 2G /swapfile && sudo chmod 600 /swapfile && \
sudo mkswap /swapfile && sudo swapon /swapfile

# 4. 重启程序 (使用 release 模式)
cargo run --release --bin integrated_ai_trader
```

---

## 总结

修复后的改进:
- ✅ **内存泄漏**: 已修复,添加自动清理
- ✅ **并发优化**: API 调用添加超时保护
- ✅ **编译冲突**: 提供智能编译脚本
- ✅ **持仓清理**: 孤立追踪器自动清理
- 📋 **系统优化**: 建议添加 Swap 分区

预期效果:
- 内存使用稳定在 2-3GB 范围内
- 不再出现 OOM Killer 事件
- 编译与运行互不干扰
