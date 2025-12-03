use anyhow::Result;
use chrono::Utc;
use log::{debug, info, warn};
use rust_trading_bot::{
    binance_client::BinanceClient, exchange_trait::ExchangeClient, trading::OrderManager,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::sync::RwLock;

use super::super::modules::types::{PositionTracker, TriggerOrderRecord};

/// 触发单监控与互斥控制
///
/// 负责周期性检查触发单状态、根据价格偏离动态取消并在止损/止盈互斥时
/// 自动清理另一方订单，避免遗留订单阻塞新策略。
pub struct TriggerMonitor {
    exchange: Arc<BinanceClient>,
    order_manager: Arc<OrderManager>,
    position_trackers: Arc<RwLock<HashMap<String, PositionTracker>>>,
    active_triggers: Arc<RwLock<HashMap<String, TriggerOrderRecord>>>,
}

impl TriggerMonitor {
    pub fn new(
        exchange: Arc<BinanceClient>,
        order_manager: Arc<OrderManager>,
        position_trackers: Arc<RwLock<HashMap<String, PositionTracker>>>,
        active_triggers: Arc<RwLock<HashMap<String, TriggerOrderRecord>>>,
    ) -> Self {
        Self {
            exchange,
            order_manager,
            position_trackers,
            active_triggers,
        }
    }

    /// 巡检触发单状态，完成、过期或偏离过大的自动取消
    pub async fn monitor_orders(&self) -> Result<()> {
        let snapshot = {
            let orders = self.active_triggers.read().await;
            if orders.is_empty() {
                return Ok(());
            }
            orders.clone()
        };

        let mut to_remove = HashSet::new();

        for (order_id, record) in snapshot.iter() {
            match self
                .exchange
                .get_order_status_detail(&record.symbol, order_id)
                .await
            {
                Ok(status) => {
                    let status_text = status.status.as_str();
                    if matches!(status_text, "FILLED" | "CANCELED" | "EXPIRED") {
                        info!("🔔 触发单 {} 已完成: {}", order_id, status.status);
                        to_remove.insert(order_id.clone());
                        continue;
                    }
                }
                Err(e) => {
                    warn!("⚠️ 查询触发单失败: {} - {}", order_id, e);
                    continue;
                }
            }

            let current_price = match self.exchange.get_current_price(&record.symbol).await {
                Ok(price) => price,
                Err(e) => {
                    warn!(
                        "⚠️ 获取 {} 当前价格失败, 暂不调整触发单 {}: {}",
                        record.symbol, order_id, e
                    );
                    continue;
                }
            };

            if self.should_cancel(record, current_price).await {
                info!(
                    "🗑️ 取消不再合理的触发单: {} @ {:.4}",
                    record.symbol, record.trigger_price
                );
                if let Err(e) = self
                    .order_manager
                    .cancel_order(&record.symbol, order_id)
                    .await
                {
                    warn!("⚠️ 取消触发单失败: {}", e);
                } else {
                    to_remove.insert(order_id.clone());
                }
            }
        }

        if !to_remove.is_empty() {
            let mut orders = self.active_triggers.write().await;
            for order_id in to_remove {
                orders.remove(&order_id);
            }
        }

        Ok(())
    }

    /// 取消指定交易对的保护单，返回被取消的 order_id
    pub async fn cancel_symbol_orders(&self, symbol: &str) -> Result<Vec<u64>> {
        let tracker_snapshot = {
            let trackers = self.position_trackers.read().await;
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
            match self.exchange.cancel_order(symbol, &order_id).await {
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
            let mut trackers = self.position_trackers.write().await;
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

    /// 清理已无持仓或长期失联的触发单
    pub async fn cleanup_orphaned(&self) -> Result<()> {
        info!("⏰ 开始执行定期孤立触发单清理...");

        let positions = self.exchange.get_positions().await?;
        let active_symbols: HashSet<String> = positions
            .iter()
            .filter(|p| p.size.abs() > f64::EPSILON)
            .map(|p| p.symbol.clone())
            .collect();

        let trackers_snapshot = {
            let trackers = self.position_trackers.read().await;
            trackers.clone()
        };

        let mut symbols_to_remove = Vec::new();
        let mut cleaned = 0usize;

        for (symbol, tracker) in trackers_snapshot {
            if active_symbols.contains(&symbol) {
                continue;
            }

            let orphaned_minutes = (Utc::now() - tracker.entry_time).num_minutes();
            debug!(
                "⏱️ {} 已空仓 {} 分钟, 开始清理遗留触发单",
                symbol, orphaned_minutes
            );

            if let Some(order_id) = tracker.stop_loss_order_id.as_deref() {
                match self.order_manager.cancel_order(&symbol, order_id).await {
                    Ok(_) => {
                        info!(
                            "🗑️ 清理孤立触发单: {} SL order_id={} (持仓已平仓)",
                            symbol, order_id
                        );
                        cleaned += 1;
                    }
                    Err(e) => {
                        warn!(
                            "⚠️ 取消孤立触发单失败: {} SL order_id={} ({})",
                            symbol, order_id, e
                        );
                    }
                }
            }

            if let Some(order_id) = tracker.take_profit_order_id.as_deref() {
                match self.order_manager.cancel_order(&symbol, order_id).await {
                    Ok(_) => {
                        info!(
                            "🗑️ 清理孤立触发单: {} TP order_id={} (持仓已平仓)",
                            symbol, order_id
                        );
                        cleaned += 1;
                    }
                    Err(e) => {
                        warn!(
                            "⚠️ 取消孤立触发单失败: {} TP order_id={} ({})",
                            symbol, order_id, e
                        );
                    }
                }
            }

            symbols_to_remove.push(symbol);
        }

        if !symbols_to_remove.is_empty() {
            let mut trackers = self.position_trackers.write().await;
            for symbol in symbols_to_remove {
                trackers.remove(&symbol);
            }
        }

        info!("✅ 定期孤立触发单清理完成 (清理 {} 个订单)", cleaned);
        Ok(())
    }

    /// 检查止盈止损互斥关系，一方成交则取消另一方
    pub async fn check_exclusion(&self) -> Result<()> {
        let trackers_snapshot: Vec<(String, Option<String>, Option<String>)> = {
            let trackers = self.position_trackers.read().await;
            trackers
                .iter()
                .filter(|(_, t)| t.stop_loss_order_id.is_some() || t.take_profit_order_id.is_some())
                .map(|(symbol, t)| {
                    (
                        symbol.clone(),
                        t.stop_loss_order_id.clone(),
                        t.take_profit_order_id.clone(),
                    )
                })
                .collect()
        };

        if trackers_snapshot.is_empty() {
            return Ok(());
        }

        let mut mutations: Vec<(String, Option<String>, Option<String>)> = Vec::new();

        for (symbol, sl_order_id, tp_order_id) in trackers_snapshot {
            let mut new_sl_id = sl_order_id.clone();
            let mut new_tp_id = tp_order_id.clone();
            let mut sl_filled = false;
            let mut tp_filled = false;

            if let Some(ref sl_id) = sl_order_id {
                match self.exchange.get_order_status(&symbol, sl_id).await {
                    Ok(status) => {
                        if matches!(status.as_str(), "FILLED" | "EXPIRED" | "CANCELED") {
                            sl_filled = status == "FILLED";
                            new_sl_id = None;
                            if sl_filled {
                                info!("🔴 {} 止损单已成交: {}", symbol, sl_id);
                            }
                        }
                    }
                    Err(e) => {
                        debug!("⚠️ {} 查询止损单状态失败: {}", symbol, e);
                        new_sl_id = None;
                    }
                }
            }

            if let Some(ref tp_id) = tp_order_id {
                match self.exchange.get_order_status(&symbol, tp_id).await {
                    Ok(status) => {
                        if matches!(status.as_str(), "FILLED" | "EXPIRED" | "CANCELED") {
                            tp_filled = status == "FILLED";
                            new_tp_id = None;
                            if tp_filled {
                                info!("🟢 {} 止盈单已成交: {}", symbol, tp_id);
                            }
                        }
                    }
                    Err(e) => {
                        debug!("⚠️ {} 查询止盈单状态失败: {}", symbol, e);
                        new_tp_id = None;
                    }
                }
            }

            if sl_filled {
                if let Some(ref tp_id) = tp_order_id {
                    match self.order_manager.cancel_order(&symbol, tp_id).await {
                        Ok(_) => info!("✅ {} 止损触发,已取消止盈单: {}", symbol, tp_id),
                        Err(e) => warn!("⚠️ {} 取消止盈单失败: {}", symbol, e),
                    }
                    new_tp_id = None;
                }
            }

            if tp_filled {
                if let Some(ref sl_id) = sl_order_id {
                    match self.order_manager.cancel_order(&symbol, sl_id).await {
                        Ok(_) => info!("✅ {} 止盈触发,已取消止损单: {}", symbol, sl_id),
                        Err(e) => warn!("⚠️ {} 取消止损单失败: {}", symbol, e),
                    }
                    new_sl_id = None;
                }
            }

            if new_sl_id != sl_order_id || new_tp_id != tp_order_id {
                mutations.push((symbol, new_sl_id, new_tp_id));
            }
        }

        if !mutations.is_empty() {
            let mut trackers = self.position_trackers.write().await;
            for (symbol, new_sl_id, new_tp_id) in mutations {
                if let Some(tracker) = trackers.get_mut(&symbol) {
                    tracker.stop_loss_order_id = new_sl_id;
                    tracker.take_profit_order_id = new_tp_id;
                }
            }
        }

        Ok(())
    }

    /// 登记新的触发单，供监控循环使用
    pub async fn register_trigger(&self, record: TriggerOrderRecord) {
        let mut orders = self.active_triggers.write().await;
        orders.insert(record.order_id.clone(), record);
    }

    async fn should_cancel(&self, order: &TriggerOrderRecord, current_price: f64) -> bool {
        let age = Utc::now() - order.created_at;
        if age.num_hours() > 4 {
            info!(
                "⏰ 触发单 {} 已挂单 {}h,自动取消",
                order.order_id,
                age.num_hours()
            );
            return true;
        }

        let trigger_price = if order.trigger_price.abs() < f64::EPSILON {
            f64::EPSILON
        } else {
            order.trigger_price
        };
        let price_deviation = ((current_price - trigger_price).abs() / trigger_price) * 100.0;

        if order.action.eq_ignore_ascii_case("OPEN") && price_deviation > 5.0 {
            info!(
                "📉 触发价 {:.4} 与当前价 {:.4} 偏离 {:.1}%,取消开仓触发单",
                order.trigger_price, current_price, price_deviation
            );
            return true;
        }

        false
    }

    /// 根据 order_id 移除触发单记录
    pub async fn remove_trigger(&self, order_id: &str) {
        let mut orders = self.active_triggers.write().await;
        orders.remove(order_id);
    }

    /// 返回所有处于监控状态的触发单快照
    pub async fn snapshot(&self) -> HashMap<String, TriggerOrderRecord> {
        let orders = self.active_triggers.read().await;
        orders.clone()
    }
}
