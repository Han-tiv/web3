use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;
use log::info;
use rust_trading_bot::{
    deepseek_client::TradingSignal, exchange_trait::Position, signals::FundAlert,
};
use tokio::sync::RwLock;

use super::super::modules::types::{PositionTracker, SignalHistory};

/// 风控中心，负责频繁交易检测及追踪币种清理。
pub struct RiskController {
    signal_history: Arc<RwLock<SignalHistory>>,
    position_trackers: Arc<RwLock<HashMap<String, PositionTracker>>>,
    tracked_coins: Arc<RwLock<HashMap<String, FundAlert>>>,
    coin_ttl_hours: i64,
    max_tracked_coins: usize,
}

impl RiskController {
    pub fn new(
        signal_history: Arc<RwLock<SignalHistory>>,
        position_trackers: Arc<RwLock<HashMap<String, PositionTracker>>>,
        tracked_coins: Arc<RwLock<HashMap<String, FundAlert>>>,
        coin_ttl_hours: i64,
        max_tracked_coins: usize,
    ) -> Self {
        Self {
            signal_history,
            position_trackers,
            tracked_coins,
            coin_ttl_hours,
            max_tracked_coins,
        }
    }

    /// 检查频繁交易或反向信号，必要时跳过当前信号。
    pub async fn check_frequent_trading(
        &self,
        signal: &TradingSignal,
        current_position: Option<&Position>,
    ) -> bool {
        if signal.signal == "HOLD" {
            return false;
        }

        if let Some(pos) = current_position {
            let is_reverse_signal = (pos.side == "LONG" && signal.signal == "SELL")
                || (pos.side == "SHORT" && signal.signal == "BUY");

            if is_reverse_signal && signal.confidence != "HIGH" {
                info!(
                    "   当前持仓: {} | 信号: {} | 信心: {}",
                    pos.side, signal.signal, signal.confidence
                );
                info!("   ⚠️  非高信心反向信号，保持现有仓位");
                return true;
            }

            if is_reverse_signal {
                let history = self.signal_history.read().await;
                let recent_signals = history.get_recent(3);
                let same_signal_count = recent_signals
                    .iter()
                    .filter(|s| s.signal == signal.signal)
                    .count();

                if same_signal_count >= 2 {
                    info!(
                        "   ⚠️  最近3次中已出现{}次{}信号，避免频繁反转",
                        same_signal_count, signal.signal
                    );
                    return true;
                }
            }
        }

        false
    }

    /// 清理过期或超出容量的追踪币种。
    pub async fn cleanup_tracked_coins(&self) -> Result<()> {
        let now = Utc::now();
        let max_tracked = self.max_tracked_coins;
        let ttl_hours = self.coin_ttl_hours;

        let mut coins = self.tracked_coins.write().await;
        coins.retain(|coin, alert| {
            let age_hours = (now - alert.timestamp).num_hours();
            if age_hours >= ttl_hours {
                info!("🗑️  清理过期币种: {} (已追踪 {} 小时)", coin, age_hours);
                false
            } else {
                true
            }
        });

        if coins.len() > max_tracked {
            let mut sorted: Vec<_> = coins
                .iter()
                .map(|(k, v)| (k.clone(), v.timestamp))
                .collect();
            sorted.sort_by_key(|(_, timestamp)| *timestamp);

            let to_remove = coins.len() - max_tracked;
            for coin in sorted.into_iter().take(to_remove) {
                if coins.remove(&coin.0).is_some() {
                    info!("🗑️  容量限制,移除最旧币种: {}", coin.0);
                }
            }
        }

        Ok(())
    }

    /// 快照当前追踪器，供其他模块读取。
    pub async fn trackers_snapshot(&self) -> HashMap<String, PositionTracker> {
        self.position_trackers.read().await.clone()
    }
}
