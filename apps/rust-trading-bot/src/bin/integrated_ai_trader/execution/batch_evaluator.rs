use std::collections::HashMap;
use std::sync::Weak;

use anyhow::Result;
use chrono::Utc;
use log::{error, info, warn};
use rust_trading_bot::exchange_trait::Position;

use super::super::{
    modules::types::{
        PositionAction, PositionContextRequest, PositionEvaluationStep, PreparedPositionContext,
        TrackerMutation,
    },
    trader::{IntegratedAITrader, TrackerSnapshot},
};

pub struct BatchEvaluator {
    trader: Weak<IntegratedAITrader>,
}

impl BatchEvaluator {
    pub fn new(trader: Weak<IntegratedAITrader>) -> Self {
        Self { trader }
    }

    /// AI批量评估持仓并执行建议的交易动作
    pub(crate) async fn evaluate(
        &self,
        tracker_snapshots: HashMap<String, TrackerSnapshot>,
        exchange_positions: &[Position],
    ) -> Result<()> {
        if tracker_snapshots.is_empty() {
            return Ok(());
        }

        let Some(trader) = self.trader.upgrade() else {
            warn!("⚠️ BatchEvaluator: 无法获取交易器实例");
            return Ok(());
        };

        let mut actions_to_execute = Vec::new();
        let mut batch_inputs = Vec::new();
        let mut batch_contexts: HashMap<String, PreparedPositionContext> = HashMap::new();

        for snapshot in tracker_snapshots.values() {
            let symbol = snapshot.symbol.clone();
            let side = snapshot.side.clone();
            let entry_price = snapshot.entry_price;
            let entry_time = snapshot.entry_time;
            let quantity = snapshot.quantity;

            let maybe_position = exchange_positions.iter().find(|p| p.symbol == symbol);

            if maybe_position.is_none() {
                info!("✅ {} 持仓已平仓(止损/止盈触发)", symbol);
                actions_to_execute.push(PositionAction::Remove(symbol));
                continue;
            }

            let position = maybe_position.unwrap();
            let current_price = position.mark_price;
            let live_quantity = position.size.abs();

            let notional_value = live_quantity * current_price;
            const MIN_NOTIONAL: f64 = 1.0;

            if notional_value < MIN_NOTIONAL {
                warn!(
                    "⚠️  {} 发现尘埃持仓 (数量={:.8}, 价格=${:.4}, 价值=${:.4}), 视为已平仓并清理",
                    symbol, live_quantity, current_price, notional_value
                );
                actions_to_execute.push(PositionAction::Remove(symbol.clone()));
                continue;
            }

            let duration = (Utc::now() - entry_time).num_minutes() as f64 / 60.0;

            let profit_pct = if side == "LONG" {
                ((current_price - entry_price) / entry_price) * 100.0
            } else {
                ((entry_price - current_price) / entry_price) * 100.0
            };

            info!(
                "📊 {} 持仓检查: 方向={} | 入场=${:.4} | 当前=${:.4} | 盈亏={:+.2}% | 时长={:.1}h",
                symbol, side, entry_price, current_price, profit_pct, duration
            );

            let duration_minutes = (Utc::now() - entry_time).num_minutes();
            if duration_minutes < 5 && profit_pct < -0.5 {
                warn!(
                    "🚨 {} 5分钟法则触发: 持仓{}分钟亏损{:.2}%, 入场失败立即止损",
                    symbol, duration_minutes, profit_pct
                );
                actions_to_execute.push(PositionAction::FullClose {
                    symbol,
                    side,
                    quantity,
                    reason: "entry_failure_5min".to_string(),
                });
                continue;
            }

            if duration >= 0.5 && profit_pct < -3.0 {
                warn!(
                    "🚨 {} 快速止损触发: {}分钟亏损{:+.2}%, 执行全仓止损",
                    symbol,
                    (duration * 60.0) as i32,
                    profit_pct
                );
                actions_to_execute.push(PositionAction::FullClose {
                    symbol,
                    side,
                    quantity,
                    reason: format!("quick_stop_loss_-3pct_{}min", (duration * 60.0) as i32),
                });
                continue;
            }

            if profit_pct < -5.0 {
                warn!(
                    "🚨 {} 亏损超过-5%({:+.2}%),执行极端止损",
                    symbol, profit_pct
                );
                actions_to_execute.push(PositionAction::FullClose {
                    symbol,
                    side,
                    quantity,
                    reason: "extreme_loss".to_string(),
                });
                continue;
            }

            let stop_loss_price = if let Some(ref sl_id) = snapshot.stop_loss_order_id {
                match trader
                    .exchange
                    .get_order_status_detail(&symbol, sl_id)
                    .await
                {
                    Ok(status) => status.stop_price.unwrap_or(entry_price),
                    Err(_) => entry_price,
                }
            } else if side == "LONG" {
                entry_price * 0.95
            } else {
                entry_price * 1.05
            };

            let req = PositionContextRequest {
                symbol: &symbol,
                side: &side,
                entry_price,
                stop_loss_price,
                current_price,
                quantity,
                duration_hours: duration,
                stop_loss_order_id: snapshot.stop_loss_order_id.clone(),
                take_profit_order_id: snapshot.take_profit_order_id.clone(),
            };

            match trader
                .position_evaluator
                .context_builder()
                .prepare_position_context(req)
                .await
            {
                Ok(PositionEvaluationStep::Immediate(action)) => {
                    actions_to_execute.push(action);
                }
                Ok(PositionEvaluationStep::Context(ctx)) => {
                    batch_inputs.push(ctx.to_batch_input().into());
                    batch_contexts.insert(ctx.symbol.clone(), ctx);
                }
                Ok(PositionEvaluationStep::Skip) => {}
                Err(e) => {
                    warn!("⚠️  {} 准备AI评估上下文失败: {}", symbol, e);
                }
            }
        }

        if !batch_inputs.is_empty() {
            match trader.gemini.evaluate_positions_batch(batch_inputs).await {
                Ok(results) => {
                    for (symbol, decision) in results {
                        if let Some(ctx) = batch_contexts.remove(&symbol) {
                            match trader
                                .position_evaluator
                                .decision_handler()
                                .handle_decision(&ctx, &decision)
                                .await
                            {
                                Ok(Some(action)) => actions_to_execute.push(action),
                                Ok(None) => {}
                                Err(e) => {
                                    warn!("⚠️  {} 应用AI决策失败: {}", symbol, e);
                                }
                            }
                        } else {
                            warn!("⚠️  批量AI返回未知symbol {}, 可能上下文已被移除", symbol);
                        }
                    }
                }
                Err(e) => {
                    warn!("⚠️  Gemini 批量评估失败: {}", e);
                }
            }
        }

        if actions_to_execute.is_empty() {
            return Ok(());
        }

        let mut tracker_mutations = Vec::new();
        let mut symbols_to_remove = Vec::new();

        for action in actions_to_execute {
            match action {
                PositionAction::FullClose { symbol, reason, .. } => {
                    match trader.close_position_fully_with_retry(&symbol, 3).await {
                        Ok(_) => {
                            symbols_to_remove.push(symbol);
                        }
                        Err(e) => {
                            error!("❌ 全部平仓失败({}): {}", reason, e);
                            trader
                                .send_critical_alert(
                                    &symbol,
                                    &format!("全部平仓失败({}): {}", reason, e),
                                )
                                .await;
                        }
                    }
                }
                PositionAction::PartialClose {
                    symbol,
                    side,
                    close_quantity,
                    close_pct,
                    stop_loss_price,
                    ..
                } => {
                    info!(
                        "📉 执行部分平仓: {} 计划数量 {:.6} ({}%)",
                        symbol, close_quantity, close_pct
                    );
                    let canceled_orders = match trader.cancel_symbol_trigger_orders(&symbol).await {
                        Ok(ids) => ids,
                        Err(e) => {
                            warn!("⚠️  清理旧触发单失败: {}", e);
                            Vec::new()
                        }
                    };
                    info!(
                        "🗑️ 取消旧触发单 {} 个: {:?}",
                        canceled_orders.len(),
                        canceled_orders
                    );

                    let actual_remaining =
                        match trader.close_position_partially(&symbol, close_pct).await {
                            Ok(remaining) => remaining,
                            Err(e) => {
                                error!("❌ 部分平仓失败: {}", e);
                                continue;
                            }
                        };

                    if actual_remaining > f64::EPSILON {
                        match trader
                            .exchange
                            .set_stop_loss(&symbol, &side, actual_remaining, stop_loss_price, None)
                            .await
                        {
                            Ok(new_sl_id) => {
                                tracker_mutations.push(TrackerMutation::QuantityAndStopLoss {
                                    symbol,
                                    new_quantity: actual_remaining,
                                    new_stop_loss_order_id: Some(new_sl_id),
                                });
                                info!("✅ 止损已根据实际剩余数量更新: {:.6}", actual_remaining);
                            }
                            Err(e) => {
                                warn!("⚠️  根据实际剩余数量移动止损失败: {}", e);
                                tracker_mutations.push(TrackerMutation::QuantityAndStopLoss {
                                    symbol,
                                    new_quantity: actual_remaining,
                                    new_stop_loss_order_id: None,
                                });
                            }
                        }
                    } else {
                        info!("✅ {} 部分平仓后已无剩余仓位，清理追踪器", symbol);
                        symbols_to_remove.push(symbol);
                    }
                }
                PositionAction::SetLimitOrder {
                    symbol,
                    side,
                    quantity,
                    limit_price,
                    ..
                } => {
                    let canceled_orders = match trader.cancel_symbol_trigger_orders(&symbol).await {
                        Ok(ids) => ids,
                        Err(e) => {
                            warn!("⚠️  清理旧触发单失败: {}", e);
                            Vec::new()
                        }
                    };
                    info!(
                        "🗑️ 取消旧触发单 {} 个: {:?}",
                        canceled_orders.len(),
                        canceled_orders
                    );

                    match trader
                        .exchange
                        .set_limit_take_profit(&symbol, &side, quantity, limit_price)
                        .await
                    {
                        Ok(order_id) => {
                            tracker_mutations.push(TrackerMutation::TakeProfitOrder {
                                symbol,
                                new_take_profit_order_id: Some(order_id),
                            });
                            info!("✅ 限价止盈单已设置 @ ${:.4}", limit_price);
                        }
                        Err(e) => {
                            error!("❌ 设置限价止盈单失败: {}", e);
                        }
                    }
                }
                PositionAction::Remove(symbol) => {
                    symbols_to_remove.push(symbol);
                }
            }
        }

        if !tracker_mutations.is_empty() || !symbols_to_remove.is_empty() {
            let mut trackers = trader.position_trackers.write().await;

            for mutation in tracker_mutations {
                match mutation {
                    TrackerMutation::QuantityAndStopLoss {
                        symbol,
                        new_quantity,
                        new_stop_loss_order_id,
                    } => {
                        if let Some(tracker) = trackers.get_mut(&symbol) {
                            tracker.quantity = new_quantity;
                            tracker.stop_loss_order_id = new_stop_loss_order_id;
                        }
                    }
                    TrackerMutation::TakeProfitOrder {
                        symbol,
                        new_take_profit_order_id,
                    } => {
                        if let Some(tracker) = trackers.get_mut(&symbol) {
                            tracker.take_profit_order_id = new_take_profit_order_id;
                        }
                    }
                }
            }

            for symbol in symbols_to_remove {
                trackers.remove(&symbol);
            }
        }

        Ok(())
    }
}
