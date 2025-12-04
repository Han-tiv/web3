# PHASE 2 参数精简重构方案

## 🎯 目标概览
- 20 个 `too_many_arguments` Clippy 警告覆盖面广，绝大多数集中在 AI Prompt 构建与集成交易流程。
- Phase 2 旨在引入统一的 Request/Context 结构体，降低函数签名复杂度、巩固可维护性，并为 Phase 3 的自动化 testing 铺路。
- 本文列出每个函数的**当前签名、目标结构体、重构后签名、调用示例**，并附带优先级建议与兼容策略。

## 🔝 优先级路线图
1. **EntryManager & Trader 执行链 (#18-#20)**：15 参数函数是最大技术债，直接影响实盘下单，必须最先治理。
2. **AI Prompt 族 (#1-#10)**：12-14 参数占 50%+ 警告，统一 `EntryPromptContext`/`PositionPromptContext` 可复用至多客户端。
3. **持仓上下文构建链 (#11 & #17)**：Evaluator → ContextBuilder 是 AI 平仓的入口，统一 `PositionContextRequest` 有助于测试。
4. **独立模块 (#12-#16)**：数据库、分析器、Risk Monitor 属于外围支撑，可在主链完成后并行推进。
5. **辅助逻辑 (#13 SupportAnalyzer)**：重构后能让 AI/量化模块共享相同的支持位分析请求体。

---

## 1. AI Prompt 函数组（#1-#10）
### 1.1 当前函数签名
```rust
// #1 src/deepseek_client/prompts/entry_v2.rs
pub fn build_entry_analysis_prompt_v2(
    symbol: &str,
    alert_type: &str,
    alert_message: &str,
    fund_type: &str,
    zone_1h_summary: &str,
    zone_15m_summary: &str,
    entry_action: &str,
    entry_reason: &str,
    klines_5m: &[Kline],
    klines_15m: &[Kline],
    klines_1h: &[Kline],
    current_price: f64,
) -> String

// #2 src/deepseek_client/prompts/position_v2.rs
pub fn build_position_management_prompt_v2(
    symbol: &str,
    side: &str,
    entry_price: f64,
    current_price: f64,
    profit_pct: f64,
    hold_duration_hours: f64,
    klines_5m: &[Kline],
    klines_15m: &[Kline],
    klines_1h: &[Kline],
    indicators: &TechnicalIndicators,
    support_text: &str,
    deviation_desc: &str,
    current_stop_loss: Option<f64>,
    current_take_profit: Option<f64>,
) -> String

// #3 src/deepseek_client/mod.rs: impl DeepSeekClient
pub fn build_entry_analysis_prompt_v2(
    &self,
    symbol: &str,
    alert_type: &str,
    alert_message: &str,
    fund_type: &str,
    zone_1h_summary: &str,
    zone_15m_summary: &str,
    entry_action: &str,
    entry_reason: &str,
    klines_5m: &[Kline],
    klines_15m: &[Kline],
    klines_1h: &[Kline],
    _current_price: f64,
) -> String

// #4 src/deepseek_client/mod.rs
pub fn build_position_management_prompt_v2(
    &self,
    symbol: &str,
    side: &str,
    entry_price: f64,
    current_price: f64,
    profit_pct: f64,
    hold_duration_hours: f64,
    klines_5m: &[Kline],
    klines_15m: &[Kline],
    klines_1h: &[Kline],
    indicators: &TechnicalIndicators,
    support_text: &str,
    deviation_desc: &str,
) -> String

// #5 src/gemini_client/prompts/entry_v2.rs
pub fn build_entry_analysis_prompt_v2(
    symbol: &str,
    alert_type: &str,
    alert_message: &str,
    fund_type: &str,
    zone_1h_summary: &str,
    zone_15m_summary: &str,
    entry_action: &str,
    entry_reason: &str,
    klines_5m: &[Kline],
    klines_15m: &[Kline],
    klines_1h: &[Kline],
    current_price: f64,
) -> String

// #6 src/gemini_client/prompts/position_v2.rs
pub fn build_position_management_prompt_v2(
    symbol: &str,
    side: &str,
    entry_price: f64,
    current_price: f64,
    profit_pct: f64,
    hold_duration_hours: f64,
    klines_5m: &[Kline],
    klines_15m: &[Kline],
    klines_1h: &[Kline],
    indicators: &TechnicalIndicators,
    support_text: &str,
    deviation_desc: &str,
    current_stop_loss: Option<f64>,
    current_take_profit: Option<f64>,
) -> String

// #7 src/gemini_client/mod.rs
pub fn build_entry_analysis_prompt_v2(
    &self,
    symbol: &str,
    alert_type: &str,
    alert_message: &str,
    fund_type: &str,
    zone_1h_summary: &str,
    zone_15m_summary: &str,
    entry_action: &str,
    entry_reason: &str,
    klines_5m: &[Kline],
    klines_15m: &[Kline],
    klines_1h: &[Kline],
    _current_price: f64,
) -> String

// #8 src/gemini_client/mod.rs
pub fn build_position_management_prompt_v2(
    &self,
    symbol: &str,
    side: &str,
    entry_price: f64,
    current_price: f64,
    profit_pct: f64,
    hold_duration_hours: f64,
    klines_5m: &[Kline],
    klines_15m: &[Kline],
    klines_1h: &[Kline],
    indicators: &TechnicalIndicators,
    support_text: &str,
    deviation_desc: &str,
) -> String

// #9 src/grok_client.rs
pub fn build_entry_analysis_prompt(
    &self,
    symbol: &str,
    alert_type: &str,
    alert_message: &str,
    fund_type: &str,
    zone_1h_summary: &str,
    zone_15m_summary: &str,
    entry_action: &str,
    entry_reason: &str,
    klines_5m: &[Kline],
    klines_15m: &[Kline],
    klines_1h: &[Kline],
    _current_price: f64,
) -> String

// #10 src/grok_client.rs
pub fn build_position_management_prompt(
    &self,
    symbol: &str,
    side: &str,
    entry_price: f64,
    current_price: f64,
    profit_pct: f64,
    hold_duration_hours: f64,
    klines_5m: &[Kline],
    klines_15m: &[Kline],
    klines_1h: &[Kline],
    indicators: &TechnicalIndicators,
    support_text: &str,
    deviation_desc: &str,
) -> String
```

### 1.2 统一结构体
现有 `EntryPromptContext<'a>` 已在 `src/bin/integrated_ai_trader/modules/types.rs` 定义，下一步需要搬迁至 `rust_trading_bot::ai::context`（或新的 `prompt_context.rs`）并适度扩展；同时补充缺失的 `PositionPromptContext<'a>`：

```rust
pub struct EntryPromptContext<'a> {
    pub symbol: &'a str,
    pub alert_type: &'a str,
    pub alert_message: &'a str,
    pub fund_type: &'a str,
    pub zone_1h_summary: &'a str,
    pub zone_15m_summary: &'a str,
    pub entry_action: &'a str,
    pub entry_reason: &'a str,
    pub klines_5m: &'a [Kline],
    pub klines_15m: &'a [Kline],
    pub klines_1h: &'a [Kline],
    pub klines_4h: Option<&'a [Kline]>,
    pub current_price: f64,
    pub change_24h: Option<f64>,
    pub signal_type: Option<&'a str>,
    pub technical_indicators: Option<&'a TechnicalIndicators>,
}

pub struct PositionPromptContext<'a> {
    pub symbol: &'a str,
    pub side: &'a str,
    pub entry_price: f64,
    pub current_price: f64,
    pub profit_pct: f64,
    pub hold_duration_hours: f64,
    pub klines_5m: &'a [Kline],
    pub klines_15m: &'a [Kline],
    pub klines_1h: &'a [Kline],
    pub indicators: &'a TechnicalIndicators,
    pub support_text: &'a str,
    pub deviation_desc: &'a str,
    pub current_stop_loss: Option<f64>,
    pub current_take_profit: Option<f64>,
}
```

两者都带 `'a` 生命周期，允许在构造上下文时借用现有切片/字符串而无需分配。

### 1.3 重构后签名
```rust
// #1/#5/#9
pub fn build_entry_analysis_prompt_v2(ctx: &EntryPromptContext<'_>) -> String;
// #3/#7
pub fn build_entry_analysis_prompt_v2(&self, ctx: &EntryPromptContext<'_>) -> String;

// #2/#6/#10
pub fn build_position_management_prompt_v2(ctx: &PositionPromptContext<'_>) -> String;
// #4/#8
pub fn build_position_management_prompt_v2(&self, ctx: &PositionPromptContext<'_>) -> String;
```

### 1.4 调用方修改示例
```rust
let entry_ctx = EntryPromptContext {
    symbol: &symbol,
    alert_type: alert_type_str,
    alert_message: &alert.raw_message,
    fund_type: &alert.fund_type,
    zone_1h_summary: &zone_1h_summary,
    zone_15m_summary: &zone_15m_summary,
    entry_action: &entry_action_str,
    entry_reason: &entry_decision.reason,
    klines_5m: &klines_5m,
    klines_15m: &klines,
    klines_1h: &klines_1h,
    klines_4h: None,
    current_price,
    change_24h: None,
    signal_type: None,
    technical_indicators: None,
};
let prompt = deepseek_client.build_entry_analysis_prompt_v2(&entry_ctx);
```

同理，`PositionEvaluator` 或 `ContextBuilder` 中构造 `PositionPromptContext` 后传给 `build_position_management_prompt_v2`，函数体内部只需改为 `ctx.symbol` 等字段。

### 1.5 风险与兼容
- **兼容层**：在 `deepseek_client/gemini_client/grok_client` 暴露的新 API 之外，可临时保留旧函数并 `#[deprecated]`，内部 `EntryPromptContext::from_legacy_args(...)`，保证未完成迁移期间不阻塞。
- **生命周期**：保持 `ctx` 仅在函数内部使用；如需长期存储（例如缓存 prompt），需额外 Clone。但 现阶段 prompt 即用即弃，可直接借用。
- **指标差异**：Gemini 版本需要 `fund_flow_text`，迁移后在函数内部基于 `ctx.alert_type` 现算即可。

---

## 2. 持仓上下文链（#11 & #17）
### 2.1 当前签名
```rust
// #11 src/bin/integrated_ai_trader/ai/context_builder.rs
pub async fn prepare_position_context(
    &self,
    symbol: &str,
    side: &str,
    entry_price: f64,
    stop_loss_price: f64,
    current_price: f64,
    quantity: f64,
    duration: f64,
    stop_loss_order_id: Option<String>,
    take_profit_order_id: Option<String>,
) -> Result<PositionEvaluationStep>

// #17 src/bin/integrated_ai_trader/ai/evaluator.rs
pub async fn evaluate(
    &self,
    symbol: &str,
    side: &str,
    entry_price: f64,
    stop_loss_price: f64,
    current_price: f64,
    quantity: f64,
    duration: f64,
    stop_loss_order_id: Option<String>,
    take_profit_order_id: Option<String>,
) -> Result<Option<PositionAction>>
```

### 2.2 新结构体：`PositionContextRequest<'a>`
```rust
pub struct PositionContextRequest<'a> {
    pub symbol: &'a str,
    pub side: &'a str,
    pub entry_price: f64,
    pub stop_loss_price: f64,
    pub current_price: f64,
    pub quantity: f64,
    pub duration_hours: f64,
    pub stop_loss_order_id: Option<String>,
    pub take_profit_order_id: Option<String>,
}
```

- 因上下文需要在 `prepare_position_context` 中多次克隆 order_id，用 `Option<String>` 持有所有权，调用方在创建 request 前执行 `clone()`，避免生命周期烦恼。

### 2.3 重构签名
```rust
pub async fn prepare_position_context(
    &self,
    req: PositionContextRequest<'_>,
) -> Result<PositionEvaluationStep>;

pub async fn evaluate(
    &self,
    req: PositionContextRequest<'_>,
) -> Result<Option<PositionAction>>;
```

### 2.4 调用示例
```rust
let req = PositionContextRequest {
    symbol,
    side,
    entry_price,
    stop_loss_price,
    current_price,
    quantity,
    duration_hours: duration,
    stop_loss_order_id: stop_loss_order_id.clone(),
    take_profit_order_id: take_profit_order_id.clone(),
};
if let Some(action) = evaluator.evaluate(req).await? { /* ... */ }
```

---

## 3. 数据与工具函数（#12-#15）
### 3.1 Database::record_trade_profit (#12)
```rust
pub fn record_trade_profit(
    &self,
    symbol: &str,
    entry_time: i64,
    exit_time: i64,
    entry_price: f64,
    exit_price: f64,
    quantity: f64,
    side: &str,
    profit_usdt: f64,
    capital_used: f64,
) -> DbResult<()>
```
**新结构体**
```rust
pub struct RecordTradeProfitParams<'a> {
    pub symbol: &'a str,
    pub entry_time: i64,
    pub exit_time: i64,
    pub entry_price: f64,
    pub exit_price: f64,
    pub quantity: f64,
    pub side: &'a str,
    pub profit_usdt: f64,
    pub capital_used: f64,
}
```
**新签名**
```rust
pub fn record_trade_profit(&self, params: &RecordTradeProfitParams<'_>) -> DbResult<()>;
```
**调用示例**
```rust
let params = RecordTradeProfitParams { /* ... */ };
db.record_trade_profit(&params)?;
```

### 3.2 SupportAnalyzer::analyze_supports (#13)
```rust
pub fn analyze_supports(
    &self,
    _klines_5m: &[Kline],
    klines_15m: &[Kline],
    klines_1h: &[Kline],
    current_price: f64,
    entry_price: f64,
    sma_20: f64,
    sma_50: f64,
    bb_lower: f64,
    bb_middle: f64,
) -> Result<SupportAnalysis>
```
**新结构体**
```rust
pub struct SupportAnalysisRequest<'a> {
    pub klines_5m: Option<&'a [Kline]>,
    pub klines_15m: &'a [Kline],
    pub klines_1h: &'a [Kline],
    pub current_price: f64,
    pub entry_price: f64,
    pub sma_20: f64,
    pub sma_50: f64,
    pub bb_lower: f64,
    pub bb_middle: f64,
}
```
**新签名**
```rust
pub fn analyze_supports(&self, req: SupportAnalysisRequest<'_>) -> Result<SupportAnalysis>;
```
**调用示例**
```rust
let req = SupportAnalysisRequest { klines_5m: Some(&support_klines_5m), /* ... */ };
let analysis = support_analyzer.analyze_supports(req)?;
```

### 3.3 SmartMoneyTracker::generate_long_signal (#14)
```rust
fn generate_long_signal(
    &self,
    current_price: f64,
    indicators: &TechnicalIndicators,
    key_levels: &[KeyLevel],
    nearest_support: Option<&KeyLevel>,
    nearest_resistance: Option<&KeyLevel>,
    money_flow_strength: f64,
    volume_ratio: f64,
    current_position: Option<&str>,
) -> Option<TradingSignal>
```
**新结构体**
```rust
pub struct LongSignalContext<'a> {
    pub current_price: f64,
    pub indicators: &'a TechnicalIndicators,
    pub key_levels: &'a [KeyLevel],
    pub nearest_support: Option<&'a KeyLevel>,
    pub nearest_resistance: Option<&'a KeyLevel>,
    pub money_flow_strength: f64,
    pub volume_ratio: f64,
    pub current_position: Option<&'a str>,
}
```
**新签名**
```rust
fn generate_long_signal(&self, ctx: &LongSignalContext<'_>) -> Option<TradingSignal>;
```
**调用示例**
```rust
let ctx = LongSignalContext { current_position: position_state.as_deref(), /* ... */ };
self.generate_long_signal(&ctx);
```

### 3.4 BinanceClient::place_trigger_order (#15)
```rust
pub async fn place_trigger_order(
    &self,
    symbol: &str,
    trigger_type: &str,
    action: &str,
    position_side: &str,
    quantity: f64,
    stop_price: f64,
    limit_price: Option<f64>,
) -> Result<String>
```
**新结构体**
```rust
pub struct TriggerOrderRequest<'a> {
    pub symbol: &'a str,
    pub trigger_type: &'a str,
    pub action: &'a str,
    pub position_side: &'a str,
    pub quantity: f64,
    pub stop_price: f64,
    pub limit_price: Option<f64>,
}
```
**新签名**
```rust
pub async fn place_trigger_order(&self, req: TriggerOrderRequest<'_>) -> Result<String>;
```
**调用示例**
```rust
let req = TriggerOrderRequest { symbol, trigger_type: "STOP", /* ... */ };
binance.place_trigger_order(req).await?;
```

---

## 4. 风控/监控函数（#16）
```rust
// src/bin/profit_monitor.rs
async fn monitor_positions(
    client: &BinanceClient,
    stop_loss_percent: f64,
    alert_percent: f64,
    leverage: u32,
    health_monitor: &HealthMonitor,
    lock_manager: &TradingLockManager,
    telegram_client: &Client,
    telegram_config: &TelegramConfig,
    auto_close_enabled: bool,
) -> Result<()>
```
**新结构体**
```rust
pub struct ProfitMonitorConfig<'a> {
    pub client: &'a BinanceClient,
    pub stop_loss_percent: f64,
    pub alert_percent: f64,
    pub leverage: u32,
    pub health_monitor: &'a HealthMonitor,
    pub lock_manager: &'a TradingLockManager,
    pub telegram_client: &'a Client,
    pub telegram_config: &'a TelegramConfig,
    pub auto_close_enabled: bool,
}
```
**新签名**
```rust
async fn monitor_positions(cfg: ProfitMonitorConfig<'_>) -> Result<()>;
```
**调用示例**
```rust
let cfg = ProfitMonitorConfig { client: &client, auto_close_enabled, /* ... */ };
monitor_positions(cfg).await?;
```

---

## 5. Entry Pipeline（#18-#20）
### 5.1 当前签名
```rust
// #18 src/bin/integrated_ai_trader/core/entry_manager.rs
pub fn new(
    exchange: Arc<BinanceClient>,
    deepseek: Arc<DeepSeekClient>,
    gemini: Arc<GeminiClient>,
    analyzer: Arc<TechnicalAnalyzer>,
    entry_zone_analyzer: Arc<EntryZoneAnalyzer>,
    staged_manager: Arc<RwLock<StagedPositionManager>>,
    position_trackers: Arc<RwLock<HashMap<String, PositionTracker>>>,
    pending_entries: Arc<RwLock<HashMap<String, PendingEntry>>>,
    signal_history: Arc<RwLock<SignalHistory>>,
    last_analysis_time: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    max_position_usdt: f64,
    min_position_usdt: f64,
    max_leverage: u32,
    min_leverage: u32,
    db: Database,
) -> Self

// #19 src/bin/integrated_ai_trader/core/entry_manager.rs
async fn execute_ai_trial_entry(
    &self,
    symbol: &str,
    alert: &FundAlert,
    zone_1h: &EntryZone,
    entry_decision: &EntryDecision,
    klines: &[Kline],
    klines_5m: &[Kline],
    current_price: f64,
    final_entry_price: f64,
    final_stop_loss: f64,
    final_confidence: &str,
    ai_position_multiplier: f64,
    ai_signal_side: &str,
    take_profit: Option<f64>,
    is_ai_override: bool,
) -> Result<()>

// #20 src/bin/integrated_ai_trader/trader_entry_executor.rs
pub(super) async fn execute_ai_trial_entry(
    &self,
    symbol: &str,
    alert: &FundAlert,
    zone_1h: &EntryZone,
    entry_decision: &EntryDecision,
    klines: &[Kline],
    klines_5m: &[Kline],
    current_price: f64,
    final_entry_price: f64,
    final_stop_loss: f64,
    final_confidence: &str,
    ai_position_multiplier: f64,
    ai_signal_side: &str,
    take_profit: Option<f64>,
    is_ai_override: bool,
) -> Result<()>
```

### 5.2 新结构体
**EntryManager::new → `EntryManagerConfig`**
```rust
pub struct EntryManagerConfig {
    pub exchange: Arc<BinanceClient>,
    pub deepseek: Arc<DeepSeekClient>,
    pub gemini: Arc<GeminiClient>,
    pub analyzer: Arc<TechnicalAnalyzer>,
    pub entry_zone_analyzer: Arc<EntryZoneAnalyzer>,
    pub staged_manager: Arc<RwLock<StagedPositionManager>>,
    pub position_trackers: Arc<RwLock<HashMap<String, PositionTracker>>>,
    pub pending_entries: Arc<RwLock<HashMap<String, PendingEntry>>>,
    pub signal_history: Arc<RwLock<SignalHistory>>,
    pub last_analysis_time: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    pub risk_limits: RiskLimitConfig,
    pub db: Database,
}

pub struct RiskLimitConfig {
    pub max_position_usdt: f64,
    pub min_position_usdt: f64,
    pub max_leverage: u32,
    pub min_leverage: u32,
}
```

**Entry 执行 → `EntryExecutionRequest<'a>`**
```rust
pub struct EntryExecutionRequest<'a> {
    pub symbol: &'a str,
    pub alert: &'a FundAlert,
    pub zone_1h: &'a EntryZone,
    pub entry_decision: &'a EntryDecision,
    pub klines_15m: &'a [Kline],
    pub klines_5m: &'a [Kline],
    pub current_price: f64,
    pub final_entry_price: f64,
    pub final_stop_loss: f64,
    pub final_confidence: &'a str,
    pub ai_position_multiplier: f64,
    pub ai_signal_side: &'a str,
    pub take_profit: Option<f64>,
    pub is_ai_override: bool,
}
```

### 5.3 重构签名
```rust
pub fn new(cfg: EntryManagerConfig) -> Self;

async fn execute_ai_trial_entry(
    &self,
    req: EntryExecutionRequest<'_>,
) -> Result<()>;
```
入口 `IntegratedAITrader` 使用相同 `EntryExecutionRequest`，避免双份实现在两个模块内漂移。

### 5.4 调用示例
```rust
let exec_req = EntryExecutionRequest {
    symbol: &symbol,
    alert: &alert,
    zone_1h: &zone_1h,
    entry_decision: &entry_decision,
    klines_15m: &klines,
    klines_5m: &klines_5m,
    current_price,
    final_entry_price,
    final_stop_loss,
    final_confidence: final_confidence.as_str(),
    ai_position_multiplier,
    ai_signal_side: normalized_ai_signal.as_str(),
    take_profit: ai_signal.take_profit,
    is_ai_override,
};
self.execute_ai_trial_entry(exec_req).await?;
```

### 5.5 风险控制
- **并发安全**：`EntryExecutionRequest` 仅存借用引用，不会跨 `await` 保存，对 `'a` 要求可通过 `Send + 'async` 检查（所有借用数据存活于 `analyze_and_trade` 函数栈内）。
- **Trader/Manager 共享实现**：`IntegratedAITrader::execute_ai_trial_entry` 可委托给 `EntryManager::execute_ai_trial_entry`，或通过 trait 抽象复用，避免逻辑漂移。
- **配置兼容**：`EntryManagerConfig::from_env()` 可封装现有 `EntryManager::new` 调用逻辑，保持构建处最小 diff。

---

## 6. 统一结构总结
- **EntryPromptContext / PositionPromptContext**：落地于共享模块，并为所有 prompt builder 使用；阶段性保留旧函数以平滑迁移。
- **PositionContextRequest**：贯穿 Evaluator → ContextBuilder → DecisionHandler → Prompt 构建，利于后续批量测试。
- **EntryManagerConfig & EntryExecutionRequest**：拆分依赖注入与业务参数，引入 `RiskLimitConfig` 让风险阈值集中管理。
- **其他 Config**：`RecordTradeProfitParams、SupportAnalysisRequest、LongSignalContext、TriggerOrderRequest、ProfitMonitorConfig` 采用 `&` 引用 + `Copy` 值组合，遵循「数据聚合 + 行为细分」的 SOLID 原则。

---

## 7. 实施顺序 & 验证建议
1. **EntryExecutionRequest**：先实现 request + builder/helper（例如 `EntryExecutionRequest::new(...)`），并迁移 `EntryManager` 与 `IntegratedAITrader`。完成后运行集成测试（试探下单 dry-run + `cargo test entry_manager::tests`）。
2. **Prompt Contexts**：自下而上替换（prompts crate → client impl → 调用方），过程中加 `#[cfg(test)]` 覆盖 prompt 输出 snapshot，确保不产生 diff。
3. **PositionContextRequest**：重构 Evaluator/ContextBuilder，同步更新 `PositionEvaluator::evaluate` 的调用者（AI Trader、profit monitor、调度任务）。
4. **数据库/工具**：采用 `Params` 结构 + `From` 实现配合 builder，减少临时代码。执行 `cargo test support_analyzer`/`smart_money_tracker`.
5. **监控与杂项**：`monitor_positions` 及 `place_trigger_order` 重构后运行 `cargo run --bin profit_monitor -- --dry-run`。

完成上述步骤后，可以将 `clippy::too_many_arguments` 允许列表清空，并开启 CI 中对同类告警的 `deny`.

---

## 8. 兼容性与后续展望
- **平滑迁移**：所有新函数在初期提供 `From<LegacyArgs>` 辅助或 `impl From<&PreparedPositionContext> for PositionPromptContext<'_>`，确保调用点重构粒度可控。
- **可测试性**：Request 结构体天然支持构造 fixture，便于在单元测试里复刻上下文而不需 `BinanceClient`/`GeminiClient` 实例。
- **生命周期管理**：统一 `'a` 策略避免 `String` clone 泛滥，同时 Request 仅在调用链短生命周期内存在，符合借用检查要求。
- **后续工作**：Phase 3 可基于 Request 结构编写序列化/日志，或导出到外部 AI 观测服务，进一步提升透明度。

> 通过以上计划，可一次性消除 20 条过多参数告警，同时为后续的模块化扩展与自动测试打下基础。

