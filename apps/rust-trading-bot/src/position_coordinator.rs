use chrono::{DateTime, Duration, Utc};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::ai_decision_engine::AiDecision;
use crate::exchange_trait::Position;

/// 仓位状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionState {
    pub symbol: String,
    pub side: String, // "LONG" or "SHORT"
    pub size: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub unrealized_pnl: f64,
    pub last_update: DateTime<Utc>,
    pub lock_until: Option<DateTime<Utc>>, // 冷却期
    pub adjustment_count: usize,           // 本周期调整次数
}

/// 交易动作类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeActionType {
    OpenLong,       // 开多
    OpenShort,      // 开空
    CloseLong,      // 平多
    CloseShort,     // 平空
    AddLong,        // 加多仓
    AddShort,       // 加空仓
    ReduceLong,     // 减多仓
    ReduceShort,    // 减空仓
    ReverseToLong,  // 反向到多
    ReverseToShort, // 反向到空
    Hold,           // 持有
}

/// 交易动作优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TradePriority {
    Low = 0,
    Medium = 1,
    High = 2,
    Emergency = 3, // 紧急止损
}

/// 交易动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeAction {
    pub symbol: String,
    pub action_type: TradeActionType,
    pub quantity: f64,
    pub priority: TradePriority,
    pub reason: String,
    pub ai_confidence: String,
    pub leverage: u32,
}

/// 决策冲突类型
#[derive(Debug, Clone)]
enum ConflictType {
    SameDirection,     // 同向信号
    OppositeDirection, // 反向信号
    NoPosition,        // 无持仓
}

/// 仓位协调器配置
#[derive(Debug, Clone)]
pub struct PositionCoordinatorConfig {
    /// 冷却期（秒）
    pub cooldown_period_secs: i64,
    /// 单周期最大调整次数
    pub max_adjustments_per_cycle: usize,
    /// 调整阈值（百分比）
    pub adjustment_threshold_pct: f64,
    /// 反向信号需要的最低置信度
    pub reverse_min_confidence: String,
}

impl Default for PositionCoordinatorConfig {
    fn default() -> Self {
        Self {
            cooldown_period_secs: 300,     // 5分钟冷却
            max_adjustments_per_cycle: 2,  // 单周期最多调整2次
            adjustment_threshold_pct: 5.0, // 5%以下不调整
            reverse_min_confidence: "HIGH".to_string(),
        }
    }
}

/// 仓位协调器
pub struct PositionCoordinator {
    config: PositionCoordinatorConfig,
    positions: Arc<RwLock<HashMap<String, PositionState>>>,
    last_ai_decisions: Arc<RwLock<HashMap<String, AiDecision>>>,
}

impl PositionCoordinator {
    pub fn new(config: PositionCoordinatorConfig) -> Self {
        Self {
            config,
            positions: Arc::new(RwLock::new(HashMap::new())),
            last_ai_decisions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 同步持仓状态（从交易所）
    pub async fn sync_positions(&self, exchange_positions: Vec<Position>) {
        let mut positions = self.positions.write().await;

        for pos in exchange_positions {
            let state = PositionState {
                symbol: pos.symbol.clone(),
                side: pos.side.clone(),
                size: pos.size,
                entry_price: pos.entry_price,
                current_price: pos.mark_price,
                unrealized_pnl: pos.pnl,
                last_update: Utc::now(),
                lock_until: None,
                adjustment_count: 0,
            };

            positions.insert(pos.symbol.clone(), state);
        }

        info!("持仓同步完成: {} 个持仓", positions.len());
    }

    /// 合并AI决策并生成交易计划
    pub async fn merge_decisions_to_plan(
        &self,
        ai_decisions: Vec<AiDecision>,
        leverage: u32,
    ) -> Vec<TradeAction> {
        let mut actions = Vec::new();
        let positions = self.positions.read().await;

        for decision in ai_decisions {
            // 检查冷却期
            if let Some(pos) = positions.get(&decision.symbol) {
                if self.is_in_cooldown(pos) {
                    info!("⏳ {} 在冷却期内，跳过", decision.symbol);
                    continue;
                }

                // 检查调整次数限制
                if pos.adjustment_count >= self.config.max_adjustments_per_cycle {
                    info!(
                        "⚠️  {} 本周期调整次数达上限 ({})",
                        decision.symbol, pos.adjustment_count
                    );
                    continue;
                }
            }

            // 根据持仓状态和AI决策生成交易动作
            let action = self.resolve_decision(&decision, &positions, leverage).await;

            if let Some(action) = action {
                actions.push(action);
            }

            // 保存AI决策
            let mut last_decisions = self.last_ai_decisions.write().await;
            last_decisions.insert(decision.symbol.clone(), decision);
        }

        // 按优先级排序
        actions.sort_by(|a, b| b.priority.cmp(&a.priority));

        info!("生成交易计划: {} 个动作", actions.len());
        actions
    }

    /// 解析单个决策
    async fn resolve_decision(
        &self,
        decision: &AiDecision,
        positions: &HashMap<String, PositionState>,
        leverage: u32,
    ) -> Option<TradeAction> {
        let signal = &decision.signal.signal;
        let confidence = &decision.signal.confidence;
        let current_position = positions.get(&decision.symbol);

        let conflict_type = self.detect_conflict(signal, current_position);

        match conflict_type {
            ConflictType::NoPosition => {
                // 无持仓，根据信号开仓
                self.handle_no_position(decision, signal, confidence, leverage)
            }

            ConflictType::SameDirection => {
                // 同向信号，考虑加仓
                self.handle_same_direction(
                    decision,
                    current_position.unwrap(),
                    confidence,
                    leverage,
                )
            }

            ConflictType::OppositeDirection => {
                // 反向信号，判断是否反手
                self.handle_opposite_direction(
                    decision,
                    current_position.unwrap(),
                    confidence,
                    leverage,
                )
            }
        }
    }

    /// 检测冲突类型
    fn detect_conflict(&self, signal: &str, position: Option<&PositionState>) -> ConflictType {
        match position {
            None => ConflictType::NoPosition,
            Some(pos) => {
                let signal_is_long = signal == "BUY";
                let position_is_long = pos.side.to_uppercase() == "LONG";

                if signal == "HOLD" || signal_is_long == position_is_long {
                    ConflictType::SameDirection
                } else {
                    ConflictType::OppositeDirection
                }
            }
        }
    }

    /// 处理无持仓情况
    fn handle_no_position(
        &self,
        decision: &AiDecision,
        signal: &str,
        confidence: &str,
        leverage: u32,
    ) -> Option<TradeAction> {
        // 低信心信号不开仓
        if confidence == "LOW" {
            debug!("{} - 低信心信号，不开仓", decision.symbol);
            return None;
        }

        match signal {
            "BUY" => Some(TradeAction {
                symbol: decision.symbol.clone(),
                action_type: TradeActionType::OpenLong,
                quantity: 0.0, // 将由执行器根据资金计算
                priority: if confidence == "HIGH" {
                    TradePriority::High
                } else {
                    TradePriority::Medium
                },
                reason: format!("AI建议开多: {}", decision.signal.reason),
                ai_confidence: confidence.to_string(),
                leverage,
            }),

            "SELL" => Some(TradeAction {
                symbol: decision.symbol.clone(),
                action_type: TradeActionType::OpenShort,
                quantity: 0.0,
                priority: if confidence == "HIGH" {
                    TradePriority::High
                } else {
                    TradePriority::Medium
                },
                reason: format!("AI建议开空: {}", decision.signal.reason),
                ai_confidence: confidence.to_string(),
                leverage,
            }),

            _ => None,
        }
    }

    /// 处理同向信号
    fn handle_same_direction(
        &self,
        decision: &AiDecision,
        position: &PositionState,
        confidence: &str,
        leverage: u32,
    ) -> Option<TradeAction> {
        // HOLD 信号保持现状
        if decision.signal.signal == "HOLD" {
            return None;
        }

        // 高信心同向信号，考虑加仓
        if confidence == "HIGH" {
            let action_type = if position.side.to_uppercase() == "LONG" {
                TradeActionType::AddLong
            } else {
                TradeActionType::AddShort
            };

            Some(TradeAction {
                symbol: decision.symbol.clone(),
                action_type,
                quantity: position.size * 0.2, // 加20%仓位
                priority: TradePriority::Medium,
                reason: format!("高信心同向信号，加仓: {}", decision.signal.reason),
                ai_confidence: confidence.to_string(),
                leverage,
            })
        } else {
            // 非高信心，保持
            None
        }
    }

    /// 处理反向信号
    fn handle_opposite_direction(
        &self,
        decision: &AiDecision,
        position: &PositionState,
        confidence: &str,
        leverage: u32,
    ) -> Option<TradeAction> {
        // 只有高信心才考虑反向
        if confidence != self.config.reverse_min_confidence {
            info!(
                "⚠️  {} 反向信号但信心不足 ({}), 保持现有仓位",
                decision.symbol, confidence
            );
            return None;
        }

        // 高信心反向信号，平仓并反手
        warn!(
            "🔄 {} 高信心反向信号，准备反手: {}",
            decision.symbol, decision.signal.reason
        );

        let (close_type, reverse_type) = if position.side.to_uppercase() == "LONG" {
            (TradeActionType::CloseLong, TradeActionType::ReverseToShort)
        } else {
            (TradeActionType::CloseShort, TradeActionType::ReverseToLong)
        };

        // 返回反向动作（包含先平仓再开仓的逻辑）
        Some(TradeAction {
            symbol: decision.symbol.clone(),
            action_type: reverse_type,
            quantity: position.size, // 记录原仓位大小，执行器会处理
            priority: TradePriority::High,
            reason: format!("高信心反向信号: {}", decision.signal.reason),
            ai_confidence: confidence.to_string(),
            leverage,
        })
    }

    /// 检查是否在冷却期
    fn is_in_cooldown(&self, position: &PositionState) -> bool {
        if let Some(lock_until) = position.lock_until {
            Utc::now() < lock_until
        } else {
            false
        }
    }

    /// 执行后更新仓位状态
    pub async fn update_position_after_trade(&self, symbol: &str, new_position: Option<Position>) {
        let mut positions = self.positions.write().await;

        match new_position {
            Some(pos) => {
                let state = positions
                    .entry(symbol.to_string())
                    .or_insert_with(|| PositionState {
                        symbol: symbol.to_string(),
                        side: pos.side.clone(),
                        size: pos.size,
                        entry_price: pos.entry_price,
                        current_price: pos.mark_price,
                        unrealized_pnl: pos.pnl,
                        last_update: Utc::now(),
                        lock_until: None,
                        adjustment_count: 0,
                    });

                // 更新状态
                state.side = pos.side;
                state.size = pos.size;
                state.entry_price = pos.entry_price;
                state.current_price = pos.mark_price;
                state.unrealized_pnl = pos.pnl;
                state.last_update = Utc::now();
                state.adjustment_count += 1;

                // 设置冷却期
                state.lock_until =
                    Some(Utc::now() + Duration::seconds(self.config.cooldown_period_secs));

                info!(
                    "✅ {} 仓位更新: {} {} @ ${:.2}",
                    symbol, state.side, state.size, state.entry_price
                );
            }

            None => {
                // 无持仓，移除
                positions.remove(symbol);
                info!("✅ {} 仓位已清空", symbol);
            }
        }
    }

    /// 重置周期调整计数器
    pub async fn reset_cycle_counters(&self) {
        let mut positions = self.positions.write().await;
        for pos in positions.values_mut() {
            pos.adjustment_count = 0;
        }
        info!("周期调整计数器已重置");
    }

    /// 获取当前所有持仓
    pub async fn get_all_positions(&self) -> Vec<PositionState> {
        let positions = self.positions.read().await;
        positions.values().cloned().collect()
    }
}
