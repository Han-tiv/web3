//! Trading State Management
//!
//! 使用 DashMap 替代 Arc<RwLock<HashMap>> 提升并发性能

use dashmap::DashMap;
use std::sync::Arc;

use crate::domain::position::Position as DomainPosition;
use crate::domain::{Order, Signal};

/// 交易状态容器
pub struct TradingState {
    /// 信号缓存
    pub signals: DashMap<String, Signal>,
    /// 持仓缓存
    pub positions: DashMap<String, DomainPosition>,
    /// 订单缓存  
    pub orders: DashMap<String, Order>,
}

impl TradingState {
    /// 创建新的状态容器
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            signals: DashMap::new(),
            positions: DashMap::new(),
            orders: DashMap::new(),
        })
    }

    // ========== 信号操作 ==========

    /// 添加信号
    pub fn add_signal(&self, symbol: String, signal: Signal) {
        self.signals.insert(symbol, signal);
    }

    /// 获取信号
    pub fn get_signal(&self, symbol: &str) -> Option<Signal> {
        self.signals.get(symbol).map(|r| r.value().clone())
    }

    /// 移除信号
    pub fn remove_signal(&self, symbol: &str) -> Option<Signal> {
        self.signals.remove(symbol).map(|(_, v)| v)
    }

    /// 获取所有待处理信号
    pub fn get_pending_signals(&self) -> Vec<Signal> {
        self.signals
            .iter()
            .filter(|entry| entry.value().is_pending())
            .map(|entry| entry.value().clone())
            .collect()
    }

    // ========== 持仓操作 ==========

    /// 添加持仓
    pub fn add_position(&self, symbol: String, position: DomainPosition) {
        self.positions.insert(symbol, position);
    }

    /// 获取持仓
    pub fn get_position(&self, symbol: &str) -> Option<DomainPosition> {
        self.positions.get(symbol).map(|r| r.value().clone())
    }

    /// 更新持仓价格
    pub fn update_position_price(&self, symbol: &str, current_price: f64) {
        if let Some(mut entry) = self.positions.get_mut(symbol) {
            entry.current_price = current_price;
            entry.calculate_pnl();
        }
    }

    /// 移除持仓
    pub fn remove_position(&self, symbol: &str) -> Option<DomainPosition> {
        self.positions.remove(symbol).map(|(_, v)| v)
    }

    /// 获取所有活跃持仓
    pub fn get_active_positions(&self) -> Vec<DomainPosition> {
        self.positions
            .iter()
            .filter(|entry| entry.value().is_active())
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// 持仓数量
    pub fn position_count(&self) -> usize {
        self.positions.len()
    }

    // ========== 订单操作 ==========

    /// 添加订单
    pub fn add_order(&self, order_id: String, order: Order) {
        self.orders.insert(order_id, order);
    }

    /// 获取订单
    pub fn get_order(&self, order_id: &str) -> Option<Order> {
        self.orders.get(order_id).map(|r| r.value().clone())
    }

    /// 移除订单
    pub fn remove_order(&self, order_id: &str) -> Option<Order> {
        self.orders.remove(order_id).map(|(_, v)| v)
    }

    /// 获取所有活跃订单
    pub fn get_active_orders(&self) -> Vec<Order> {
        self.orders
            .iter()
            .filter(|entry| entry.value().is_active())
            .map(|entry| entry.value().clone())
            .collect()
    }

    // ========== 清理操作 ==========

    /// 清理所有已关闭持仓
    pub fn cleanup_closed_positions(&self) -> usize {
        let to_remove: Vec<String> = self
            .positions
            .iter()
            .filter(|entry| !entry.value().is_active())
            .map(|entry| entry.key().clone())
            .collect();

        let count = to_remove.len();
        for key in to_remove {
            self.positions.remove(&key);
        }
        count
    }

    /// 清理所有已完成订单
    pub fn cleanup_finished_orders(&self) -> usize {
        let to_remove: Vec<String> = self
            .orders
            .iter()
            .filter(|entry| !entry.value().is_active())
            .map(|entry| entry.key().clone())
            .collect();

        let count = to_remove.len();
        for key in to_remove {
            self.orders.remove(&key);
        }
        count
    }
}

impl Default for TradingState {
    fn default() -> Self {
        Self {
            signals: DashMap::new(),
            positions: DashMap::new(),
            orders: DashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{PositionSide, PositionStatus, SignalSource, SignalType};

    #[test]
    fn test_trading_state_signals() {
        let state = TradingState::default();

        let signal = Signal::new(
            "BTCUSDT".to_string(),
            SignalType::FundInflow,
            SignalSource::Telegram,
            "Test".to_string(),
        );

        state.add_signal("BTCUSDT".to_string(), signal.clone());
        assert!(state.get_signal("BTCUSDT").is_some());

        let pending = state.get_pending_signals();
        assert_eq!(pending.len(), 1);
    }
}
