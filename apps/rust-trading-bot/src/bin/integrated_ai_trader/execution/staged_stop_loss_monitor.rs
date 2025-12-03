use std::sync::Weak;

use anyhow::Result;
use chrono::Utc;
use log::{error, info, warn};

use super::super::{
    modules::types::{PositionAction, PositionContextRequest},
    trader::IntegratedAITrader,
    utils::{converters::timestamp_ms_to_datetime, validators::is_meme_coin},
};

pub struct StagedStopLossMonitor {
    trader: Weak<IntegratedAITrader>,
}

impl StagedStopLossMonitor {
    pub fn new(trader: Weak<IntegratedAITrader>) -> Self {
        Self { trader }
    }

    /// 检查分批持仓的快速止损和AI动态止盈
    pub async fn monitor(&self) -> Result<()> {
        let Some(trader) = self.trader.upgrade() else {
            warn!("⚠️ StagedStopLossMonitor: 无法获取交易器实例");
            return Ok(());
        };

        let staged_manager = trader.staged_manager.read().await;
        let all_positions: Vec<String> = staged_manager.positions.keys().cloned().collect();
        drop(staged_manager);

        for symbol in all_positions {
            let current_price = match trader.exchange.get_current_price(&symbol).await {
                Ok(price) => price,
                Err(e) => {
                    warn!("⚠️  获取{}当前价格失败: {}", symbol, e);
                    continue;
                }
            };

            // 获取持仓时长 - trial_entry_time 是 i64 毫秒时间戳
            let staged_manager_read = trader.staged_manager.read().await;
            let duration_hours = if let Some(position) = staged_manager_read.positions.get(&symbol)
            {
                let now_ms = Utc::now().timestamp_millis();
                let duration_ms = now_ms - position.trial_entry_time;
                (duration_ms as f64) / 3600000.0
            } else {
                0.0
            };
            drop(staged_manager_read);

            let staged_manager = trader.staged_manager.read().await;
            match staged_manager.check_stop_loss(&symbol, current_price, duration_hours) {
                Ok(Some(reason)) => {
                    info!("🚨 {} 触发快速止损: {}", symbol, reason);

                    let (_side, _quantity) =
                        if let Some(position) = staged_manager.positions.get(&symbol) {
                            (position.side.clone(), position.total_quantity)
                        } else {
                            drop(staged_manager);
                            continue;
                        };

                    drop(staged_manager);
                    match trader.close_position_fully_with_retry(&symbol, 3).await {
                        Ok(_) => info!("✅ 快速止损平仓成功: {}", symbol),
                        Err(e) => {
                            error!("❌ 快速止损平仓失败: {}", e);
                            trader
                                .send_critical_alert(
                                    &symbol,
                                    &format!("快速止损执行失败: {} - {}", reason, e),
                                )
                                .await;
                        }
                    }
                }
                Ok(None) => {
                    drop(staged_manager);

                    // ✅ 即使不触发硬性止损,也让AI评估是否应该动态止盈
                    let staged_snapshot = {
                        let staged_manager_read = trader.staged_manager.read().await;
                        staged_manager_read.positions.get(&symbol).cloned()
                    };

                    let Some(position) = staged_snapshot else {
                        continue;
                    };

                    let side = position.side.clone();
                    let entry_price = position.avg_cost;
                    let quantity = position.total_quantity;
                    let entry_time = timestamp_ms_to_datetime(position.trial_entry_time);
                    let duration = (Utc::now() - entry_time).num_minutes() as f64 / 60.0;
                    let profit_pct = if side == "LONG" {
                        ((current_price - entry_price) / entry_price) * 100.0
                    } else {
                        ((entry_price - current_price) / entry_price) * 100.0
                    };

                    let is_meme = is_meme_coin(&symbol);
                    let mut forced_stop_reason: Option<String> = None;

                    if is_meme && duration >= 1.0 && profit_pct <= -2.0 {
                        forced_stop_reason =
                            Some("MEME币60分钟亏损超过2%，触发硬性止损".to_string());
                    } else if is_meme && duration >= 2.0 {
                        forced_stop_reason = Some("MEME币持仓超过2小时，触发时间止损".to_string());
                    } else if !is_meme && duration >= 2.0 && profit_pct <= -3.0 {
                        forced_stop_reason =
                            Some("持仓超过2小时且亏损3%，触发保守退出".to_string());
                    } else if !is_meme && duration >= 4.0 && profit_pct <= 0.0 {
                        forced_stop_reason = Some("持仓超过4小时未盈利，触发保守退出".to_string());
                    }

                    if profit_pct <= -5.0 {
                        forced_stop_reason = Some("亏损超过5%，触发极端防守止损".to_string());
                    }

                    if duration >= 0.5 && profit_pct <= -3.0 {
                        forced_stop_reason = Some("30分钟亏损超过3%，触发快速止损".to_string());
                    }

                    if let Some(reason) = forced_stop_reason {
                        info!("🚨 {} 硬性止损触发: {}", symbol, reason);
                        match trader.close_position_fully_with_retry(&symbol, 3).await {
                            Ok(_) => info!("✅ 硬性止损平仓成功，移除持仓记录"),
                            Err(e) => {
                                error!("❌ 硬性止损平仓失败: {}", e);
                                trader
                                    .send_critical_alert(
                                        &symbol,
                                        &format!("硬性止损执行失败: {} - {}", reason, e),
                                    )
                                    .await;
                            }
                        }
                        continue;
                    }

                    info!(
                        "🤖 {} 分批持仓AI评估: 盈亏{:+.2}%, 时长{:.1}h",
                        symbol, profit_pct, duration
                    );

                    let stop_loss_price = if side == "LONG" {
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
                        stop_loss_order_id: None,
                        take_profit_order_id: None,
                    };

                    // 使用批量评估API（即使只有1个持仓，统一使用批量接口）
                    let eval_step = match trader
                        .position_evaluator
                        .context_builder()
                        .prepare_position_context(req)
                        .await
                    {
                        Ok(step) => step,
                        Err(e) => {
                            warn!("⚠️  分批持仓准备评估上下文失败: {}", e);
                            continue;
                        }
                    };

                    let ai_action = match eval_step {
                        super::super::modules::types::PositionEvaluationStep::Immediate(action) => {
                            Some(action)
                        }
                        super::super::modules::types::PositionEvaluationStep::Skip => None,
                        super::super::modules::types::PositionEvaluationStep::Context(ctx) => {
                            let batch_input = vec![ctx.to_batch_input().into()];
                            match trader.gemini.evaluate_positions_batch(batch_input).await {
                                Ok(results) => {
                                    if let Some((_, decision)) = results.into_iter().next() {
                                        match trader
                                            .position_evaluator
                                            .decision_handler()
                                            .handle_decision(&ctx, &decision)
                                            .await
                                        {
                                            Ok(action) => action,
                                            Err(e) => {
                                                warn!("⚠️  分批持仓AI决策处理失败: {}", e);
                                                None
                                            }
                                        }
                                    } else {
                                        None
                                    }
                                }
                                Err(e) => {
                                    warn!("⚠️  Gemini批量评估失败: {}", e);
                                    None
                                }
                            }
                        }
                    };

                    match ai_action
                    {
                        Some(PositionAction::FullClose {
                            symbol: close_symbol,
                            ..
                        }) => match trader
                            .close_position_fully_with_retry(&close_symbol, 3)
                            .await
                        {
                            Ok(_) => {
                                info!("✅ 分批持仓AI平仓成功: {}", close_symbol);
                            }
                            Err(e) => {
                                error!("❌ 分批持仓AI平仓失败: {}", e);
                                trader
                                    .send_critical_alert(
                                        &close_symbol,
                                        &format!("分批持仓AI建议全平但执行失败: {}", e),
                                    )
                                    .await;
                            }
                        },
                        Some(PositionAction::PartialClose {
                            symbol: close_symbol,
                            close_pct,
                            ..
                        }) => {
                            info!(
                                "📉 分批持仓AI建议部分平仓 {} ({}%)",
                                close_symbol, close_pct
                            );
                            match trader
                                .close_position_partially(&close_symbol, close_pct)
                                .await
                            {
                                Ok(remaining_qty) => {
                                    let mut staged_manager = trader.staged_manager.write().await;
                                    if let Some(position) =
                                        staged_manager.positions.get_mut(&close_symbol)
                                    {
                                        position.total_quantity = remaining_qty.max(0.0);
                                        info!(
                                            "✅ 分批持仓数量已同步: {:.6}",
                                            position.total_quantity
                                        );
                                        if position.total_quantity <= 0.0001 {
                                            staged_manager.positions.remove(&close_symbol);
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("❌ 分批持仓AI部分平仓失败: {}", e);
                                }
                            }
                        }
                        Some(PositionAction::SetLimitOrder { .. }) => {
                            warn!("⚠️  分批持仓暂不支持AI限价止盈同步,保持持仓");
                        }
                        Some(PositionAction::Remove(_)) => {}
                        None => {}
                    }
                }
                Err(e) => {
                    warn!("⚠️  {} 止损检查失败: {}", symbol, e);
                    drop(staged_manager);
                }
            }
        }

        Ok(())
    }
}
