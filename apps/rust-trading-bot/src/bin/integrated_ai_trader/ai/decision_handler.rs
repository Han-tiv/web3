use anyhow::Result;
use chrono::Utc;
use log::{info, warn};
use rust_trading_bot::{
    database::{AiAnalysisRecord, Database},
    deepseek_client::PositionManagementDecision,
};
use std::sync::Arc;

use super::super::utils::converters::{map_confidence_to_score, normalize_signal_type};
use super::super::{PositionAction, PreparedPositionContext};

/// 负责将 AI 决策落地为交易动作并记录分析日志
#[derive(Clone)]
pub struct DecisionHandler {
    db: Arc<Database>,
}

impl DecisionHandler {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn handle_decision(
        &self,
        ctx: &PreparedPositionContext,
        ai_decision: &PositionManagementDecision,
    ) -> Result<Option<PositionAction>> {
        info!(
            "🎯 AI 决策: {} | 理由: {} | 盈利潜力: {} | 置信度: {}",
            ai_decision.action,
            ai_decision.reason,
            ai_decision.profit_potential,
            ai_decision.confidence
        );

        let confidence_value = map_confidence_to_score(&ai_decision.confidence);
        let decision_text = format!(
            "{} | 盈利潜力: {} | 置信度: {}",
            ai_decision.action, ai_decision.profit_potential, ai_decision.confidence
        );
        let signal_type = normalize_signal_type(&ai_decision.action);
        let ai_record = AiAnalysisRecord {
            id: None,
            timestamp: Utc::now().to_rfc3339(),
            symbol: ctx.symbol.clone(),
            decision: decision_text,
            confidence: confidence_value,
            signal_type: Some(signal_type.to_string()),
            reason: ai_decision.reason.clone(),
            valuescan_score: None,
            risk_reward_ratio: None,
            entry_price: None,
            stop_loss: None,
            resistance: None,
            support: None,
        };

        if let Err(e) = self.db.insert_ai_analysis(&ai_record) {
            warn!("⚠️  保存AI持仓分析到数据库失败: {}", e);
        }

        let params = ActionBuildParams {
            symbol: ctx.symbol.clone(),
            side: ctx.side.clone(),
            entry_price: ctx.entry_price,
            stop_loss_price: ctx.stop_loss_price,
            quantity: ctx.quantity,
            stop_loss_order_id: ctx.stop_loss_order_id.clone(),
            take_profit_order_id: ctx.take_profit_order_id.clone(),
            min_notional: ctx.min_notional,
            current_price: ctx.current_price,
        };

        Ok(Self::build_action(ai_decision, params))
    }

    pub fn build_action(
        decision: &PositionManagementDecision,
        params: ActionBuildParams,
    ) -> Option<PositionAction> {
        match decision.action.as_str() {
            "HOLD" => {
                info!("✅ AI 建议继续持有 {}", params.symbol);
                None
            }
            "PARTIAL_CLOSE" => {
                let close_pct = decision.close_percentage.unwrap_or(50.0);
                if decision.close_percentage.is_none() {
                    warn!("⚠️  AI 建议部分平仓但未提供百分比,使用默认50%");
                }
                info!("📉 AI 建议部分平仓 {} ({}%)", params.symbol, close_pct);
                let close_quantity =
                    (params.quantity * (close_pct / 100.0)).clamp(0.0, params.quantity);
                let remaining_quantity = (params.quantity - close_quantity).max(0.0);

                if close_quantity <= f64::EPSILON {
                    warn!("⚠️  计算得到的平仓数量过小, 跳过本次部分平仓");
                    None
                } else {
                    let position_total_value = params.quantity * params.current_price;
                    let suggested_close_value = close_quantity * params.current_price;

                    if suggested_close_value < params.min_notional {
                        let min_ratio_pct =
                            (params.min_notional / position_total_value * 100.0).ceil();

                        if min_ratio_pct <= 100.0 {
                            let adjusted_close_pct = min_ratio_pct;
                            let adjusted_close_qty = params.quantity * (adjusted_close_pct / 100.0);
                            let adjusted_close_value = adjusted_close_qty * params.current_price;

                            warn!(
                                "⚠️ {} 部分平仓比率调整: AI建议{:.0}% (${:.2}) → 实际执行{:.0}% (${:.2})，满足MIN_NOTIONAL ${:.0}",
                                params.symbol,
                                close_pct,
                                suggested_close_value,
                                adjusted_close_pct,
                                adjusted_close_value,
                                params.min_notional
                            );

                            let adjusted_remaining =
                                (params.quantity - adjusted_close_qty).max(0.0);
                            Some(PositionAction::PartialClose {
                                symbol: params.symbol.clone(),
                                side: params.side.clone(),
                                close_quantity: adjusted_close_qty,
                                close_pct: adjusted_close_pct,
                                entry_price: params.entry_price,
                                stop_loss_price: params.stop_loss_price,
                                remaining_quantity: adjusted_remaining,
                                stop_loss_order_id: params.stop_loss_order_id.clone(),
                            })
                        } else {
                            warn!(
                                "⚠️ {} 持仓总价值(${:.2}) < MIN_NOTIONAL(${:.0})，无法部分平仓，执行全部平仓",
                                params.symbol, position_total_value, params.min_notional
                            );
                            Some(PositionAction::FullClose {
                                symbol: params.symbol.clone(),
                                side: params.side.clone(),
                                quantity: params.quantity,
                                reason: "min_notional_full_close".to_string(),
                            })
                        }
                    } else {
                        Some(PositionAction::PartialClose {
                            symbol: params.symbol.clone(),
                            side: params.side.clone(),
                            close_quantity,
                            close_pct,
                            entry_price: params.entry_price,
                            stop_loss_price: params.stop_loss_price,
                            remaining_quantity,
                            stop_loss_order_id: params.stop_loss_order_id.clone(),
                        })
                    }
                }
            }
            "FULL_CLOSE" => {
                info!("🚨 AI 建议全部平仓 {}", params.symbol);
                Some(PositionAction::FullClose {
                    symbol: params.symbol.clone(),
                    side: params.side.clone(),
                    quantity: params.quantity,
                    reason: "ai_decision".to_string(),
                })
            }
            "SET_LIMIT_ORDER" => {
                if let Some(limit_price) = decision.limit_price {
                    info!(
                        "🎯 AI 建议设置限价止盈单 {} @ ${:.4}",
                        params.symbol, limit_price
                    );
                    Some(PositionAction::SetLimitOrder {
                        symbol: params.symbol.clone(),
                        side: params.side.clone(),
                        quantity: params.quantity,
                        limit_price,
                        take_profit_order_id: params.take_profit_order_id.clone(),
                    })
                } else {
                    warn!("⚠️  AI 建议设置限价单但未提供价格,保持持仓");
                    None
                }
            }
            other => {
                warn!("⚠️  未知的 AI 决策动作: {}, 保持持仓", other);
                None
            }
        }
    }
}

/// 构建交易动作所需的所有上下文
#[derive(Clone)]
pub struct ActionBuildParams {
    pub symbol: String,
    pub side: String,
    pub entry_price: f64,
    pub stop_loss_price: f64,
    pub quantity: f64,
    pub stop_loss_order_id: Option<String>,
    pub take_profit_order_id: Option<String>,
    pub min_notional: f64,
    pub current_price: f64,
}
