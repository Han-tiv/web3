/// 分批建仓管理模块
///
/// 核心功能:
/// 1. 管理试探期仓位(15%-30%)
/// 2. 检测加仓时机(启动信号确认)
/// 3. 计算平均成本
/// 4. 动态调整止损
use crate::entry_zone_analyzer::EntryDecision;
use crate::launch_signal_detector::LaunchSignal;
use anyhow::Result;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 仓位状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PositionStage {
    NoPosition,    // 无仓位
    TrialPosition, // 试探期(15%-30%仓位)
    FullPosition,  // 满仓期(100%仓位)
}

/// 分批仓位信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagedPosition {
    pub symbol: String,
    pub side: String, // "LONG" or "SHORT"
    pub stage: PositionStage,

    // 试探期信息
    pub trial_entry_price: f64, // 试探入场价
    pub trial_quantity: f64,    // 试探数量
    pub trial_entry_time: i64,  // 试探入场时间(毫秒)
    pub trial_stop_loss: f64,   // 试探期止损

    // 加仓信息
    pub add_entry_price: f64, // 加仓入场价(0表示未加仓)
    pub add_quantity: f64,    // 加仓数量
    pub add_entry_time: i64,  // 加仓时间(毫秒,0表示未加仓)

    // 综合信息
    pub avg_cost: f64,       // 平均成本
    pub total_quantity: f64, // 总数量
    pub full_stop_loss: f64, // 满仓期止损

    // 入场区信息
    pub entry_decision: String, // 入场决策原因
}

impl StagedPosition {
    /// 创建试探期仓位
    pub fn new_trial(
        symbol: String,
        side: String,
        entry_price: f64,
        quantity: f64,
        stop_loss: f64,
        decision_reason: String,
    ) -> Self {
        Self {
            symbol,
            side,
            stage: PositionStage::TrialPosition,
            trial_entry_price: entry_price,
            trial_quantity: quantity,
            trial_entry_time: chrono::Utc::now().timestamp_millis(),
            trial_stop_loss: stop_loss,
            add_entry_price: 0.0,
            add_quantity: 0.0,
            add_entry_time: 0,
            avg_cost: entry_price,
            total_quantity: quantity,
            full_stop_loss: stop_loss,
            entry_decision: decision_reason,
        }
    }

    /// 执行加仓
    pub fn add_position(&mut self, add_price: f64, add_qty: f64, new_stop_loss: f64) {
        self.add_entry_price = add_price;
        self.add_quantity = add_qty;
        self.add_entry_time = chrono::Utc::now().timestamp_millis();

        // 重新计算平均成本
        self.total_quantity = self.trial_quantity + self.add_quantity;
        self.avg_cost = (self.trial_entry_price * self.trial_quantity
            + self.add_entry_price * self.add_quantity)
            / self.total_quantity;

        // 更新止损
        self.full_stop_loss = new_stop_loss;

        // 更新状态
        self.stage = PositionStage::FullPosition;

        info!(
            "✅ {} 加仓完成: 试探{:.4}@${:.4} + 加仓{:.4}@${:.4} = 平均成本${:.4}",
            self.symbol,
            self.trial_quantity,
            self.trial_entry_price,
            self.add_quantity,
            self.add_entry_price,
            self.avg_cost
        );
    }

    /// 获取当前盈亏百分比
    pub fn get_profit_pct(&self, current_price: f64) -> f64 {
        if self.side == "LONG" {
            ((current_price - self.avg_cost) / self.avg_cost) * 100.0
        } else {
            ((self.avg_cost - current_price) / self.avg_cost) * 100.0
        }
    }

    /// 获取持仓时长(小时)
    pub fn get_hold_duration_hours(&self) -> f64 {
        let start_time = self.trial_entry_time;
        let now = chrono::Utc::now().timestamp_millis();
        ((now - start_time) as f64) / 3600000.0
    }
}

/// 分批建仓管理器
pub struct StagedPositionManager {
    pub positions: HashMap<String, StagedPosition>, // symbol -> position
    pub add_position_ratio: f64,                    // 0.7 (加仓70%)
}

impl Default for StagedPositionManager {
    fn default() -> Self {
        Self {
            positions: HashMap::new(),
            add_position_ratio: 0.7,
        }
    }
}

impl StagedPositionManager {
    /// 新建试探期仓位
    pub fn create_trial_position(
        &mut self,
        symbol: String,
        side: String,
        entry_decision: &EntryDecision,
        available_usdt: f64,
        leverage: f64,
    ) -> Result<StagedPosition> {
        if self.positions.contains_key(&symbol) {
            anyhow::bail!("❌ {} 已存在仓位,不能重复建仓", symbol);
        }

        // 计算试探期数量
        let position_value = available_usdt * entry_decision.position; // 仓位价值(USDT)
        let quantity = (position_value * leverage) / entry_decision.price; // 数量

        info!(
            "🎯 {} 创建试探期仓位: 价格${:.4}, 仓位{:.0}%, 数量{:.4}, 止损${:.4}",
            symbol,
            entry_decision.price,
            entry_decision.position * 100.0,
            quantity,
            entry_decision.stop_loss
        );

        let position = StagedPosition::new_trial(
            symbol.clone(),
            side,
            entry_decision.price,
            quantity,
            entry_decision.stop_loss,
            entry_decision.reason.clone(),
        );

        self.positions.insert(symbol.clone(), position.clone());

        Ok(position)
    }

    /// 检查是否应该加仓
    pub fn should_add_position(&self, symbol: &str, launch_signal: &LaunchSignal) -> Result<bool> {
        let position = self
            .positions
            .get(symbol)
            .ok_or_else(|| anyhow::anyhow!("❌ {} 仓位不存在", symbol))?;

        // 只有试探期才能加仓
        if position.stage != PositionStage::TrialPosition {
            return Ok(false);
        }

        // 检查启动信号是否全部确认
        if !launch_signal.all_confirmed {
            info!(
                "⏳ {} 启动信号未全部确认,继续观察 (得分{:.0}/100)",
                symbol, launch_signal.score
            );
            return Ok(false);
        }

        info!(
            "🚀 {} 启动信号全部确认!准备加仓 (得分{:.0}/100)",
            symbol, launch_signal.score
        );

        Ok(true)
    }

    /// 执行加仓
    pub fn execute_add_position(
        &mut self,
        symbol: &str,
        current_price: f64,
        available_usdt: f64,
        leverage: f64,
    ) -> Result<()> {
        let position = self
            .positions
            .get_mut(symbol)
            .ok_or_else(|| anyhow::anyhow!("❌ {} 仓位不存在", symbol))?;

        if position.stage != PositionStage::TrialPosition {
            anyhow::bail!("❌ {} 不在试探期,无法加仓", symbol);
        }

        // 计算加仓数量(70%仓位)
        let add_position_value = available_usdt * self.add_position_ratio;
        let add_quantity = (add_position_value * leverage) / current_price;

        // 计算新止损(加仓价-2%)
        let new_stop_loss = current_price * 0.98;

        position.add_position(current_price, add_quantity, new_stop_loss);

        info!(
            "✅ {} 加仓完成: 新止损${:.4}, 总仓位{:.4}",
            symbol, new_stop_loss, position.total_quantity
        );

        Ok(())
    }

    /// 检查是否触发止损
    pub fn check_stop_loss(
        &self,
        symbol: &str,
        current_price: f64,
        duration_hours: f64,
    ) -> Result<Option<String>> {
        let position = self
            .positions
            .get(symbol)
            .ok_or_else(|| anyhow::anyhow!("❌ {} 仓位不存在", symbol))?;

        let profit_pct = position.get_profit_pct(current_price);

        // 【快速止损】30分钟和60分钟检查点
        if duration_hours >= 0.5 && duration_hours < 1.0 {
            if profit_pct < -1.5 {
                return Ok(Some(format!(
                    "⏰ {} 入场30分钟亏损{:+.2}%,不是主升浪,执行快速止损",
                    symbol, profit_pct
                )));
            }
        } else if duration_hours >= 1.0 && duration_hours < 1.5 {
            if profit_pct < -2.0 {
                return Ok(Some(format!(
                    "⏰ {} 入场60分钟亏损{:+.2}%,主升浪失败,执行快速止损",
                    symbol, profit_pct
                )));
            } else if profit_pct < 0.0 {
                return Ok(Some(format!(
                    "⏰ {} 入场60分钟仍未盈利({:+.2}%),启动失败,执行时间止损",
                    symbol, profit_pct
                )));
            }
        }

        // 【兜底止损】4小时未盈利
        if duration_hours >= 4.0 && profit_pct < 1.0 {
            return Ok(Some(format!(
                "⏰ {} 超时4小时且未盈利({:+.2}%),执行兜底止损",
                symbol, profit_pct
            )));
        }

        // 【价格止损】跌破止损价
        let stop_loss = if position.stage == PositionStage::FullPosition {
            position.full_stop_loss
        } else {
            position.trial_stop_loss
        };

        if (position.side == "LONG" && current_price < stop_loss)
            || (position.side == "SHORT" && current_price > stop_loss)
        {
            return Ok(Some(format!(
                "🚨 {} 破位止损: 当前${:.4} < 止损${:.4}",
                symbol, current_price, stop_loss
            )));
        }

        Ok(None)
    }

    /// 移除仓位
    pub fn remove_position(&mut self, symbol: &str) {
        if self.positions.remove(symbol).is_some() {
            info!("🗑️ {} 仓位已移除", symbol);
        }
    }

    /// 获取仓位
    pub fn get_position(&self, symbol: &str) -> Option<&StagedPosition> {
        self.positions.get(symbol)
    }

    /// 获取所有仓位
    pub fn get_all_positions(&self) -> Vec<&StagedPosition> {
        self.positions.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_trial_position() {
        let mut manager = StagedPositionManager::default();

        let decision = EntryDecision {
            action: EntryAction::EnterNow,
            price: 0.50,
            position: 0.30,
            stop_loss: 0.48,
            reason: "Test entry".to_string(),
        };

        let position = manager
            .create_trial_position(
                "PONDUSDT".to_string(),
                "LONG".to_string(),
                &decision,
                1000.0, // 1000 USDT
                10.0,   // 10x杠杆
            )
            .unwrap();

        assert_eq!(position.stage, PositionStage::TrialPosition);
        assert_eq!(position.trial_entry_price, 0.50);
        assert_eq!(position.trial_stop_loss, 0.48);
        // 数量 = (1000 * 0.30 * 10) / 0.50 = 6000
        assert!((position.trial_quantity - 6000.0).abs() < 0.1);
    }

    #[test]
    fn test_add_position() {
        let mut position = StagedPosition::new_trial(
            "PONDUSDT".to_string(),
            "LONG".to_string(),
            0.485,
            3000.0,
            0.473,
            "Trial entry".to_string(),
        );

        position.add_position(0.498, 7000.0, 0.488);

        assert_eq!(position.stage, PositionStage::FullPosition);
        assert_eq!(position.total_quantity, 10000.0);
        // 平均成本 = (0.485 * 3000 + 0.498 * 7000) / 10000 = 0.4941
        assert!((position.avg_cost - 0.4941).abs() < 0.0001);
    }

    #[test]
    fn test_profit_calculation() {
        let position = StagedPosition::new_trial(
            "PONDUSDT".to_string(),
            "LONG".to_string(),
            0.50,
            1000.0,
            0.48,
            "Test".to_string(),
        );

        let profit_pct = position.get_profit_pct(0.55);
        assert!((profit_pct - 10.0).abs() < 0.1); // (0.55 - 0.50) / 0.50 = 10%
    }
}
