use anyhow::Result;
use chrono::Utc;
use log::{error, info, warn};
use rust_trading_bot::{
    config::database::Database,
    deepseek_client::Kline,
    entry_zone_analyzer::{Confidence, EntryDecision, EntryZone},
    signals::FundAlert,
    staged_position_manager::StagedPositionManager,
    technical_analysis::TechnicalAnalyzer,
    BinanceClient,
};
use std::{collections::HashMap, sync::Arc};
use teloxide::Bot as TelegramBot;
use tokio::sync::RwLock;

use super::super::{
    modules::types::{
        PendingEntry, PositionAction, PositionTracker, SignalHistory, TrackerMutation,
    },
    utils::{
        converters::{convert_ai_klines_to_market, convert_market_indicators_to_ai},
        validators::validate_entry_zone,
    },
};

use super::{
    order_manager::OrderManager,
    position_closer::{CloseParams, PartialCloseParams, PositionCloser},
    trigger_monitor::TriggerMonitor,
};

pub struct OrderExecutor {
    exchange: Arc<BinanceClient>,
    #[allow(dead_code)] // 预留给后续更细化的下单策略
    order_manager: Arc<OrderManager>,
    #[allow(dead_code)] // 未来直接访问数据库写扩展分析
    db: Arc<Database>,
    #[allow(dead_code)] // 预留 Telegram 推送能力
    telegram_bot: Option<Arc<TelegramBot>>,
    position_closer: Arc<PositionCloser>,
}

impl OrderExecutor {
    pub fn new(
        exchange: Arc<BinanceClient>,
        order_manager: Arc<OrderManager>,
        db: Arc<Database>,
        telegram_bot: Option<Arc<TelegramBot>>,
        position_closer: Arc<PositionCloser>,
    ) -> Self {
        Self {
            exchange,
            order_manager,
            db,
            telegram_bot,
            position_closer,
        }
    }

    pub async fn execute_trial_entry(&self, params: TrialEntryParams) -> Result<()> {
        let TrialEntryParams {
            symbol,
            alert,
            zone_1h,
            entry_decision,
            klines,
            klines_5m,
            current_price,
            final_entry_price,
            final_stop_loss,
            final_confidence,
            ai_position_multiplier,
            ai_signal_side,
            take_profit_price,
            is_ai_override,
            min_position_usdt,
            max_position_usdt,
            min_leverage,
            max_leverage,
            analyzer,
            pending_entries,
            staged_manager,
            position_trackers,
            signal_history,
            trigger_monitor,
        } = params;

        info!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("💰 第4步: 执行试探建仓");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let side = if ai_signal_side.eq_ignore_ascii_case("SELL") {
            "SHORT"
        } else {
            "LONG"
        };
        let mut stop_loss_order_id: Option<String> = None;
        let mut take_profit_order_id: Option<String> = None;

        let (position_usdt, leverage) = match zone_1h.confidence {
            Confidence::High => (max_position_usdt, max_leverage),
            Confidence::Medium => {
                let mid_position = (min_position_usdt + max_position_usdt) / 2.0;
                let mid_leverage = (min_leverage + max_leverage) / 2;
                (mid_position, mid_leverage)
            }
            Confidence::Low => (min_position_usdt, min_leverage),
        };

        let adjusted_position = entry_decision.position * ai_position_multiplier;
        let rules = self.exchange.get_symbol_trading_rules(&symbol).await?;
        let min_notional = rules.min_notional.unwrap_or(5.0);
        let base_notional = position_usdt * leverage as f64 * adjusted_position;
        let required_notional = if base_notional < min_notional {
            let adjusted = min_notional * 1.05;
            warn!(
                "📊 {} 基础名义金额 {:.2} USDT < 最低要求 {:.2} USDT, 自动提升到 {:.2} USDT",
                symbol, base_notional, min_notional, adjusted
            );
            adjusted
        } else {
            base_notional
        };
        let trial_quantity = required_notional / final_entry_price;

        info!("💰 试探建仓配置:");
        info!(
            "   AI信心度: {} → 仓位调整: {:.0}%",
            final_confidence,
            adjusted_position * 100.0
        );
        info!("   投入USDT: {:.2}", position_usdt);
        info!("   杠杆倍数: {}x", leverage);
        info!("   开仓数量: {:.6} {}", trial_quantity, alert.coin);
        info!("   入场价格: ${:.4} (AI微调)", final_entry_price);
        info!("   止损价格: ${:.4} (AI微调)", final_stop_loss);

        let signal_price = klines_5m.last().map(|k| k.close).unwrap_or(current_price);
        let entry_zone = (zone_1h.entry_range.0, zone_1h.entry_range.1);
        let market_klines = convert_ai_klines_to_market(&klines);
        let market_indicators = analyzer.calculate_indicators(&market_klines);
        let indicators = convert_market_indicators_to_ai(&market_indicators, &klines);

        if !validate_entry_zone(
            signal_price,
            final_entry_price,
            entry_zone,
            &indicators,
            is_ai_override,
        )
        .await?
        {
            warn!("⚠️ 入场区验证失败，跳过建仓");

            let symbol_owned = symbol.clone();
            let mut pending = pending_entries.write().await;
            if let Some(existing) = pending.get_mut(&symbol) {
                existing.retry_count += 1;
                existing.last_analysis_time = Utc::now();
                existing.reject_reason = "价格不在入场区".to_string();
                let retry_count = existing.retry_count;
                drop(pending);
                info!(
                    "📝 {} 已在延迟队列中，更新重试次数: {}",
                    symbol, retry_count
                );
            } else {
                pending.insert(
                    symbol_owned.clone(),
                    PendingEntry {
                        symbol: symbol_owned.clone(),
                        first_signal_time: Utc::now(),
                        last_analysis_time: Utc::now(),
                        alert: alert.clone(),
                        reject_reason: "价格不在入场区".to_string(),
                        retry_count: 0,
                        fund_escape_detected_at: None,
                    },
                );
                drop(pending);
                info!("📝 已加入延迟开仓队列: {} (价格不符)", symbol);
            }

            return Ok(());
        }

        info!("✅ 入场区验证通过，继续执行建仓");

        info!(
            "⚙️  设置交易模式: 杠杆={}x, 保证金=逐仓, 模式=单向",
            leverage
        );
        if let Err(e) = self
            .exchange
            .ensure_trading_modes(&symbol, leverage, "ISOLATED", false)
            .await
        {
            error!("❌ 设置交易模式失败: {}", e);
            return Err(e);
        }

        let order_side = if side == "LONG" { "BUY" } else { "SELL" };
        match self
            .exchange
            .limit_order(
                &symbol,
                trial_quantity,
                order_side,
                final_entry_price,
                Some(side),
                false,
            )
            .await
        {
            Ok(order_id) => {
                info!("✅ 试探建仓订单已提交: {}", order_id);

                let canceled_orders = if let Some(monitor) = trigger_monitor.as_ref() {
                    match monitor.cancel_symbol_orders(&symbol).await {
                        Ok(ids) => ids,
                        Err(e) => {
                            warn!("⚠️  清理旧触发单失败: {}", e);
                            Vec::new()
                        }
                    }
                } else {
                    Vec::new()
                };
                info!(
                    "🗑️ 取消旧触发单 {} 个: {:?}",
                    canceled_orders.len(),
                    canceled_orders
                );

                match self
                    .exchange
                    .set_stop_loss(&symbol, side, trial_quantity, final_stop_loss, None)
                    .await
                {
                    Ok(sl_order_id) => {
                        info!(
                            "✅ 止损单已设置 @ ${:.4}, 订单ID: {}",
                            final_stop_loss, sl_order_id
                        );
                        stop_loss_order_id = Some(sl_order_id);
                    }
                    Err(e) => {
                        warn!("⚠️  止损单设置失败: {}", e);
                    }
                }

                if let Some(tp_price) = take_profit_price {
                    match self
                        .exchange
                        .set_limit_take_profit(&symbol, side, trial_quantity, tp_price)
                        .await
                    {
                        Ok(tp_order_id) => {
                            info!(
                                "✅ 止盈单已设置 @ ${:.4}, 订单ID: {}",
                                tp_price, tp_order_id
                            );
                            take_profit_order_id = Some(tp_order_id);
                        }
                        Err(e) => {
                            warn!("⚠️  止盈单设置失败: {}", e);
                        }
                    }
                } else {
                    info!("ℹ️  AI未提供止盈价,不设置止盈挂单");
                }

                {
                    let mut pending = pending_entries.write().await;
                    if pending.remove(&symbol).is_some() {
                        info!("✅ {} 成功开仓，已从延迟队列移除", symbol);
                    }
                }

                let mut adjusted_entry_decision = entry_decision.clone();
                adjusted_entry_decision.price = final_entry_price;
                adjusted_entry_decision.stop_loss = final_stop_loss;
                adjusted_entry_decision.position = adjusted_position;

                let mut staged_writer = staged_manager.write().await;
                if let Err(e) = staged_writer.create_trial_position(
                    symbol.clone(),
                    side.to_string(),
                    &adjusted_entry_decision,
                    position_usdt,
                    leverage as f64,
                ) {
                    error!("❌ 创建试探持仓记录失败: {}", e);
                } else {
                    info!("✅ 试探持仓已记录,等待启动信号补仓70%");

                    let mut trackers = position_trackers.write().await;
                    trackers.insert(
                        symbol.clone(),
                        PositionTracker {
                            symbol: symbol.clone(),
                            entry_price: final_entry_price,
                            quantity: trial_quantity,
                            leverage,
                            side: side.to_string(),
                            stop_loss_order_id: stop_loss_order_id.clone(),
                            take_profit_order_id: take_profit_order_id.clone(),
                            entry_time: Utc::now(),
                            last_check_time: Utc::now(),
                        },
                    );
                    info!("✅ 持仓已同步到AI监控系统 (双轨记录)");
                }

                let mut history = signal_history.write().await;
                history.add(super::super::modules::types::SignalRecord {
                    timestamp: Utc::now().to_rfc3339(),
                    signal: if side == "LONG" {
                        "BUY".to_string()
                    } else {
                        "SELL".to_string()
                    },
                    confidence: "MEDIUM".to_string(),
                    reason: format!("试探建仓: {}", entry_decision.reason.clone()),
                    price: entry_decision.price,
                });
            }
            Err(e) => {
                error!("❌ 试探建仓订单提交失败: {}", e);
            }
        }

        Ok(())
    }

    pub async fn execute_actions(
        &self,
        _symbol: &str,
        actions: Vec<PositionAction>,
        context: &ActionExecutionContext,
    ) -> Result<()> {
        if actions.is_empty() {
            return Ok(());
        }

        let mut tracker_mutations = Vec::new();
        let mut symbols_to_remove = Vec::new();

        for action in actions {
            match action {
                PositionAction::FullClose {
                    symbol: close_symbol,
                    reason,
                    ..
                } => {
                    let params = CloseParams {
                        symbol: close_symbol.clone(),
                        max_retries: 3,
                        reason: Some(reason.clone()),
                    };
                    match self.position_closer.close_fully_with_retry(params).await {
                        Ok(_) => {
                            symbols_to_remove.push(close_symbol.clone());
                        }
                        Err(e) => {
                            error!("❌ 全部平仓失败({}): {}", reason, e);
                            let _ = self
                                .position_closer
                                .send_alert(
                                    &close_symbol,
                                    &format!("全部平仓失败({}): {}", reason, e),
                                )
                                .await;
                        }
                    }
                }
                PositionAction::PartialClose {
                    symbol: close_symbol,
                    side,
                    close_pct,
                    stop_loss_price,
                    ..
                } => {
                    info!("📉 执行部分平仓: {} 计划比例 {}%", close_symbol, close_pct);
                    let canceled_orders = match context
                        .trigger_monitor
                        .cancel_symbol_orders(&close_symbol)
                        .await
                    {
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

                    let actual_remaining = match self
                        .position_closer
                        .close_partially(PartialCloseParams::new(&close_symbol, close_pct))
                        .await
                    {
                        Ok(remaining) => remaining,
                        Err(e) => {
                            error!("❌ 部分平仓失败: {}", e);
                            continue;
                        }
                    };

                    if actual_remaining > f64::EPSILON {
                        match self
                            .exchange
                            .set_stop_loss(
                                &close_symbol,
                                &side,
                                actual_remaining,
                                stop_loss_price,
                                None,
                            )
                            .await
                        {
                            Ok(new_sl_id) => {
                                tracker_mutations.push(TrackerMutation::QuantityAndStopLoss {
                                    symbol: close_symbol.clone(),
                                    new_quantity: actual_remaining,
                                    new_stop_loss_order_id: Some(new_sl_id),
                                });
                                info!("✅ 止损已根据实际剩余数量更新: {:.6}", actual_remaining);
                            }
                            Err(e) => {
                                warn!("⚠️  根据实际剩余数量移动止损失败: {}", e);
                                tracker_mutations.push(TrackerMutation::QuantityAndStopLoss {
                                    symbol: close_symbol.clone(),
                                    new_quantity: actual_remaining,
                                    new_stop_loss_order_id: None,
                                });
                            }
                        }
                    } else {
                        info!("✅ {} 部分平仓后已无剩余仓位，清理追踪器", close_symbol);
                        symbols_to_remove.push(close_symbol.clone());
                    }

                    let mut staged_writer = context.staged_manager.write().await;
                    if let Some(position) = staged_writer.positions.get_mut(&close_symbol) {
                        position.total_quantity = actual_remaining.max(0.0);
                        if position.total_quantity <= 0.0001 {
                            staged_writer.positions.remove(&close_symbol);
                        }
                    }
                }
                PositionAction::SetLimitOrder {
                    symbol: target_symbol,
                    side,
                    quantity,
                    limit_price,
                    ..
                } => {
                    let canceled_orders = match context
                        .trigger_monitor
                        .cancel_symbol_orders(&target_symbol)
                        .await
                    {
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

                    match self
                        .exchange
                        .set_limit_take_profit(&target_symbol, &side, quantity, limit_price)
                        .await
                    {
                        Ok(order_id) => {
                            tracker_mutations.push(TrackerMutation::TakeProfitOrder {
                                symbol: target_symbol.clone(),
                                new_take_profit_order_id: Some(order_id),
                            });
                            info!("✅ 限价止盈单已设置 @ ${:.4}", limit_price);
                        }
                        Err(e) => {
                            error!("❌ 设置限价止盈单失败: {}", e);
                        }
                    }
                }
                PositionAction::Remove(clean_symbol) => {
                    symbols_to_remove.push(clean_symbol);
                }
            }
        }

        if !tracker_mutations.is_empty() || !symbols_to_remove.is_empty() {
            let mut trackers = context.position_trackers.write().await;

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

pub struct TrialEntryParams {
    pub symbol: String,
    pub alert: FundAlert,
    pub zone_1h: EntryZone,
    pub entry_decision: EntryDecision,
    pub klines: Vec<Kline>,
    pub klines_5m: Vec<Kline>,
    pub current_price: f64,
    pub final_entry_price: f64,
    pub final_stop_loss: f64,
    pub final_confidence: String,
    pub ai_position_multiplier: f64,
    pub ai_signal_side: String,
    pub take_profit_price: Option<f64>,
    pub is_ai_override: bool,
    pub min_position_usdt: f64,
    pub max_position_usdt: f64,
    pub min_leverage: u32,
    pub max_leverage: u32,
    pub analyzer: Arc<TechnicalAnalyzer>,
    pub pending_entries: Arc<RwLock<HashMap<String, PendingEntry>>>,
    pub staged_manager: Arc<RwLock<StagedPositionManager>>,
    pub position_trackers: Arc<RwLock<HashMap<String, PositionTracker>>>,
    pub signal_history: Arc<RwLock<SignalHistory>>,
    pub trigger_monitor: Option<Arc<TriggerMonitor>>,
}

pub struct ActionExecutionContext {
    pub trigger_monitor: Arc<TriggerMonitor>,
    pub position_trackers: Arc<RwLock<HashMap<String, PositionTracker>>>,
    pub staged_manager: Arc<RwLock<StagedPositionManager>>,
}
