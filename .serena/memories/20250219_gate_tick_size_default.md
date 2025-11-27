## 2025-02-19
- Gate 交易所 TradingRules 构造新增 tick_size 字段，默认值 0.0001，确保与统一交易规则结构对齐。
- cargo check 暗示其他交易所客户端仍缺少 tick_size 初始化，后续需要补齐以恢复编译。