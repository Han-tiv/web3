## 集成 AI Trader 持仓同步修复
- monitor_positions 逻辑调整：先获取实时持仓并更新 Web 状态，然后再根据 tracker_snapshots 决定是否继续 AI 管理。
- 即使 position_trackers 为空，也会同步真实持仓，确保前端显示包含手动建仓或重启后的仓位。
- 若 tracker 数据缺失则跳过 AI 决策，避免误操作，仅保留数据同步。