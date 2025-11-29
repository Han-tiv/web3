# 🚀 快速通道重构计划

**策略**: 先搭建所有模块框架，再逐步完善实现  
**优势**: 快速看到整体架构，降低复杂度  

---

## 📋 模块清单

### 1. entry_analyzer.rs - 入场分析 ⏳
```rust
- analyze_and_trade()           // 主入场分析逻辑
- check_signal_deduplication()  // 信号去重
- fetch_klines()                // 获取K线
- call_ai_analysis()            // 调用AI分析
```

### 2. entry_executor.rs - 入场执行 ⏳
```rust
- execute_ai_trial_entry()      // 执行试探开仓
- calculate_position_size()     // 计算仓位
- place_entry_order()           // 下单
- set_stop_loss_take_profit()   // 设置止盈止损
```

### 3. position_operator.rs - 持仓操作 ⏳
```rust
- close_position_fully()        // 全仓平仓
- close_position_partially()    // 部分平仓
- update_stop_loss()            // 更新止损
- cancel_orders()               // 取消订单
```

### 4. cleanup_manager.rs - 清理管理 ⏳
```rust
- cleanup_tracked_coins()       // 清理追踪币种
- cleanup_orphaned_trackers()   // 清理孤立追踪器
- cleanup_trigger_orders()      // 清理触发单
```

### 5. order_monitor.rs - 订单监控 ⏳
```rust
- monitor_trigger_orders()      // 监控触发单
- check_sl_tp_mutual_exclusion() // 止盈止损互斥检查
```

### 6. position_monitor.rs - 持仓监控 ⏳
```rust
- run()                         // 主监控循环
- check_trial_positions()       // 检查试探持仓
- check_staged_stop_loss()      // 检查分批止损
- batch_evaluate_positions()    // 批量AI评估
- execute_actions()             // 执行操作
```

### 7. position_evaluator.rs - AI评估 ⏳
```rust
- evaluate_position_with_ai()   // AI评估单个持仓
- build_evaluation_prompt()     // 构建评估提示
- parse_ai_response()           // 解析AI响应
```

---

## 执行策略

**Phase A**: 创建所有模块框架 (30分钟)
- 每个模块只实现基本结构
- 核心函数用TODO标记
- 确保可以编译通过

**Phase B**: 逐个完善实现 (2-3小时)
- 按优先级完善每个模块
- 每完成一个模块就测试
- 保持可编译状态

**Phase C**: 集成测试 (30分钟)
- 整体编译验证
- 运行测试
- 修复问题

**总计**: 3-4小时完成所有代码重构
