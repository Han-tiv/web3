use std::sync::Weak;

use anyhow::{anyhow, Result};
use chrono::Utc;
use log::{info, warn};
use rust_trading_bot::{deepseek_client::ActionParams, exchange_trait::ExchangeClient};

use super::super::{
    modules::{config::DEFAULT_VOLATILITY_PERCENT, types::TriggerOrderRecord},
    trader::IntegratedAITrader,
};

pub struct ActionExecutor {
    trader: Weak<IntegratedAITrader>,
}

impl ActionExecutor {
    pub fn new(trader: Weak<IntegratedAITrader>) -> Self {
        Self { trader }
    }

    /// 规范化side字段为(order_side, position_side)
    fn normalize_sides(side: Option<&String>) -> (Option<String>, Option<String>) {
        side.map(|value| {
            let normalized = value.trim().to_uppercase();
            match normalized.as_str() {
                "LONG" => (Some("BUY".to_string()), Some("LONG".to_string())),
                "SHORT" => (Some("SELL".to_string()), Some("SHORT".to_string())),
                "BUY" => (Some("BUY".to_string()), Some("LONG".to_string())),
                "SELL" => (Some("SELL".to_string()), Some("SHORT".to_string())),
                _ => (Some(normalized.clone()), Some(normalized)),
            }
        })
        .unwrap_or((None, None))
    }

    /// 解析订单ID列表
    fn parse_order_ids(raw: Option<&String>) -> Vec<String> {
        raw.map(|ids| {
            ids.split(|c| matches!(c, ',' | '|' | ';'))
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
    }

    /// 执行单个推荐动作
    pub async fn execute_single_action(
        &self,
        action_type: &str,
        params: ActionParams,
        current_symbol: &str,
        reason: String,
    ) -> Result<String> {
        let Some(trader) = self.trader.upgrade() else {
            return Err(anyhow!("Trader已销毁"));
        };

        let ActionParams {
            symbol,
            side,
            quantity,
            price,
            stop_loss,
            take_profit,
            auto_set_protection: _,
            trigger_price,
            order_id,
        } = params;

        let symbol = symbol
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| current_symbol.to_string());

        match action_type {
            "IMMEDIATE_CLOSE" => {
                let qty = quantity.ok_or_else(|| anyhow!("立即平仓缺少 quantity"))?;
                let (_, position_side) = Self::normalize_sides(side.as_ref());
                let position_side = position_side.ok_or_else(|| anyhow!("立即平仓缺少持仓方向"))?;

                warn!("⚠️ 立即平仓: {} - {}", symbol, reason);
                if let Err(e) = trader.close_position_fully_with_retry(&symbol, 3).await {
                    trader
                        .send_critical_alert(&symbol, &format!("立即平仓失败 ({}): {}", reason, e))
                        .await;
                    return Err(e);
                }

                Ok(format!(
                    "⚠️ 立即平仓完成: {} {} 数量 {:.4}",
                    symbol, position_side, qty
                ))
            }
            "LIMIT_ORDER" => {
                let qty = quantity.ok_or_else(|| anyhow!("限价单缺少 quantity"))?;
                let px = price.ok_or_else(|| anyhow!("限价单缺少 price"))?;
                let (order_side, position_side) = Self::normalize_sides(side.as_ref());
                let order_side = order_side.ok_or_else(|| anyhow!("限价单缺少交易方向"))?;

                let order_id = trader
                    .exchange
                    .limit_order(
                        &symbol,
                        qty,
                        &order_side,
                        px,
                        position_side.as_deref(),
                        false,
                    )
                    .await?;
                info!("📝 限价单已挂: {} {} @ {:.4}", symbol, order_side, px);

                let attachments = if stop_loss.is_some() || take_profit.is_some() {
                    let pos_side = position_side
                        .clone()
                        .ok_or_else(|| anyhow!("设置止盈止损缺少 positionSide"))?;
                    trader
                        .order_manager
                        .place_protection_orders(&symbol, &pos_side, qty, stop_loss, take_profit)
                        .await?
                } else {
                    Vec::new()
                };

                let mut message = format!(
                    "📝 限价单已挂: {} {} @ {:.4} (order_id={})",
                    symbol, order_side, px, order_id
                );
                if !attachments.is_empty() {
                    message.push_str(&format!(" | {}", attachments.join(", ")));
                }
                Ok(message)
            }
            "TRIGGER_ORDER" => {
                let qty = quantity.ok_or_else(|| anyhow!("触发单缺少 quantity"))?;
                let trigger = trigger_price.ok_or_else(|| anyhow!("触发单缺少 trigger_price"))?;
                let (_, position_side) = Self::normalize_sides(side.as_ref());
                let position_side =
                    position_side.ok_or_else(|| anyhow!("触发单缺少 position_side"))?;

                let mut action = "OPEN".to_string();
                let mut smart_close_hint: Option<String> = None;

                match trader.exchange.get_positions().await {
                    Ok(positions) => {
                        if let Some(position) = positions
                            .into_iter()
                            .find(|p| p.symbol == symbol && p.size.abs() > f64::EPSILON)
                        {
                            if position.side.eq_ignore_ascii_case(&position_side) {
                                match trader.exchange.get_current_price(&symbol).await {
                                    Ok(current_price) => {
                                        let (reason_label, should_close) =
                                            match position.side.as_str() {
                                                "LONG" => {
                                                    if trigger < current_price {
                                                        ("LONG 持仓止损判定", true)
                                                    } else if trigger > current_price {
                                                        ("LONG 持仓止盈判定", true)
                                                    } else {
                                                        ("LONG 持仓价位触发", true)
                                                    }
                                                }
                                                "SHORT" => {
                                                    if trigger > current_price {
                                                        ("SHORT 持仓止损判定", true)
                                                    } else if trigger < current_price {
                                                        ("SHORT 持仓止盈判定", true)
                                                    } else {
                                                        ("SHORT 持仓价位触发", true)
                                                    }
                                                }
                                                _ => ("", false),
                                            };

                                        if should_close {
                                            action = "CLOSE".to_string();
                                            smart_close_hint = Some(format!(
                                                "{}: 当前价={:.4} → 触发价={:.4}",
                                                reason_label, current_price, trigger
                                            ));
                                        }
                                    }
                                    Err(err) => {
                                        warn!(
                                            "⚠️  获取{}当前价失败(触发单智能判定): {}",
                                            symbol, err
                                        );
                                    }
                                }
                            }
                        }
                    }
                    Err(err) => {
                        warn!("⚠️  获取{}持仓失败(触发单智能判定): {}", symbol, err);
                    }
                }

                let volatility = match trader.calculate_volatility(&symbol).await {
                    Ok(value) => value,
                    Err(err) => {
                        warn!(
                            "⚠️  计算{}波动率失败: {}，使用默认值 {:.2}%",
                            symbol, err, DEFAULT_VOLATILITY_PERCENT
                        );
                        DEFAULT_VOLATILITY_PERCENT
                    }
                };

                let (trigger_type, limit_price_adjusted): (&str, Option<f64>) =
                    if let Some(limit) = price {
                        info!("📊 AI 指定限价 {:.4}, 使用 STOP 限价触发单", limit);
                        ("STOP", Some(limit))
                    } else if volatility > 3.0 {
                        info!("📊 市场波动率 {:.2}% (高),使用 STOP_MARKET", volatility);
                        ("STOP_MARKET", None)
                    } else if volatility < 1.0 {
                        info!("📊 市场波动率 {:.2}% (低),使用 STOP 限价单", volatility);
                        let buffer = if position_side == "LONG" {
                            1.002
                        } else {
                            0.998
                        };
                        ("STOP", Some(trigger * buffer))
                    } else {
                        info!("📊 市场波动率 {:.2}% (中),使用 STOP_MARKET", volatility);
                        ("STOP_MARKET", None)
                    };

                let order_id = trader
                    .exchange
                    .place_trigger_order(
                        &symbol,
                        trigger_type,
                        &action,
                        &position_side,
                        qty,
                        trigger,
                        limit_price_adjusted,
                    )
                    .await?;

                if let Some(hint) = &smart_close_hint {
                    info!("🤖 智能平仓判定: {}", hint);
                }

                info!(
                    "🎯 触发单已设: {} {} {} @ trigger={:.4} (type={}, order_id={})",
                    symbol, action, position_side, trigger, trigger_type, order_id
                );

                {
                    let mut orders = trader.active_trigger_orders.lock().await;
                    orders.push(TriggerOrderRecord {
                        order_id: order_id.clone(),
                        symbol: symbol.clone(),
                        position_side: position_side.clone(),
                        trigger_price: trigger,
                        action: action.clone(),
                        created_at: Utc::now(),
                        reason: reason.clone(),
                    });
                }
                info!(
                    "📒 已加入触发单监控: {} {} {} (order_id={})",
                    symbol, action, position_side, order_id
                );

                let mut message = format!(
                    "🎯 触发单已设: {} {} {} @ {:.4} (order_id={})",
                    symbol, action, position_side, trigger, order_id
                );
                if let Some(hint) = smart_close_hint {
                    message.push_str(&format!(" | {}", hint));
                }
                Ok(message)
            }
            "CANCEL_TRIGGER" => {
                let order_id = order_id
                    .as_deref()
                    .ok_or_else(|| anyhow!("取消触发单缺少 order_id"))?
                    .to_string();
                trader
                    .order_manager
                    .cancel_order(&symbol, &order_id)
                    .await?;
                {
                    let mut orders = trader.active_trigger_orders.lock().await;
                    let before = orders.len();
                    orders.retain(|record| record.order_id != order_id);
                    if before != orders.len() {
                        info!("🗂️ 已从触发单监控移除: {}", order_id);
                    }
                }
                info!("❌ 已取消触发单: {}", order_id);
                Ok(format!("❌ 已取消触发单: {}", order_id))
            }
            "SET_STOP_LOSS_TAKE_PROFIT" => {
                let qty = quantity.ok_or_else(|| anyhow!("设置止盈止损缺少 quantity"))?;
                let (_, position_side) = Self::normalize_sides(side.as_ref());
                let position_side =
                    position_side.ok_or_else(|| anyhow!("设置止盈止损缺少 positionSide"))?;

                let canceled_orders = match self.cancel_symbol_trigger_orders(&symbol).await {
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

                let mut updates = Vec::new();
                if let Some(stop_loss) = stop_loss {
                    let order_id = trader
                        .exchange
                        .set_stop_loss(&symbol, &position_side, qty, stop_loss, None)
                        .await?;
                    updates.push(format!("SL {:.4}#{}", stop_loss, order_id));
                }
                if let Some(take_profit) = take_profit {
                    let order_id = trader
                        .exchange
                        .set_limit_take_profit(&symbol, &position_side, qty, take_profit)
                        .await?;
                    updates.push(format!("TP {:.4}#{}", take_profit, order_id));
                }

                if updates.is_empty() {
                    return Err(anyhow!("未提供止损或止盈参数"));
                }

                info!("🛡️ 止盈止损已更新: {}", updates.join(", "));
                Ok(format!(
                    "🛡️ 止盈止损已更新: {} -> {}",
                    symbol,
                    updates.join(", ")
                ))
            }
            "CANCEL_STOP_LOSS_TAKE_PROFIT" => {
                let order_ids = Self::parse_order_ids(order_id.as_ref());
                if order_ids.is_empty() {
                    return Err(anyhow!("取消止盈止损缺少 order_id"));
                }
                trader
                    .order_manager
                    .cancel_orders_batch(&symbol, &order_ids)
                    .await?;
                info!("🗑️ 已取消止盈止损单: {}", order_ids.join(", "));
                Ok(format!("🗑️ 已取消止盈止损单: {}", order_ids.join(", ")))
            }
            other => Err(anyhow!("未知动作类型: {}", other)),
        }
    }

    pub async fn cancel_symbol_trigger_orders(&self, symbol: &str) -> Result<Vec<u64>> {
        let Some(trader) = self.trader.upgrade() else {
            return Err(anyhow!("Trader已销毁"));
        };

        let tracker_snapshot = {
            let trackers = trader.position_trackers.read().await;
            trackers.get(symbol).cloned()
        };

        let Some(tracker) = tracker_snapshot else {
            return Ok(Vec::new());
        };

        let mut targets: Vec<(&str, String)> = Vec::new();
        if let Some(order_id) = tracker.stop_loss_order_id.clone() {
            targets.push(("止损", order_id));
        }
        if let Some(order_id) = tracker.take_profit_order_id.clone() {
            targets.push(("止盈", order_id));
        }

        if targets.is_empty() {
            return Ok(Vec::new());
        }

        let mut canceled_raw: Vec<String> = Vec::new();

        for (order_type, order_id) in targets {
            match trader.exchange.cancel_order(symbol, &order_id).await {
                Ok(_) => {
                    info!("🧹 {} 旧{}单已取消: {}", symbol, order_type, order_id);
                    canceled_raw.push(order_id);
                }
                Err(err) => {
                    warn!(
                        "⚠️  {} 旧{}单取消失败 (order_id={}): {}",
                        symbol, order_type, order_id, err
                    );
                }
            }
        }

        if canceled_raw.is_empty() {
            return Ok(Vec::new());
        }

        {
            let mut trackers = trader.position_trackers.write().await;
            if let Some(tracker) = trackers.get_mut(symbol) {
                if tracker
                    .stop_loss_order_id
                    .as_deref()
                    .map(|id| canceled_raw.iter().any(|raw| raw == id))
                    .unwrap_or(false)
                {
                    tracker.stop_loss_order_id = None;
                }
                if tracker
                    .take_profit_order_id
                    .as_deref()
                    .map(|id| canceled_raw.iter().any(|raw| raw == id))
                    .unwrap_or(false)
                {
                    tracker.take_profit_order_id = None;
                }
                tracker.last_check_time = Utc::now();
            }
        }

        let mut canceled_numeric = Vec::new();
        for raw in canceled_raw {
            match raw.parse::<u64>() {
                Ok(id) => canceled_numeric.push(id),
                Err(_) => {
                    warn!(
                        "⚠️  order_id 无法转换为数字 (symbol={}, raw={})，仍视为已清理",
                        symbol, raw
                    );
                }
            }
        }

        Ok(canceled_numeric)
    }
}
