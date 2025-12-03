use anyhow::Result;
use chrono::Utc;
use log::{debug, info, warn};
use rust_trading_bot::exchange_trait::ExchangeClient;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::super::modules::types::PositionTracker;

/// 追踪器管理器
///
/// 负责封装 `position_trackers` 的所有并发操作，确保状态同步、清理与查询
/// 逻辑都集中于同一入口，后续迁移更安全。
pub struct TrackerManager {
    trackers: Arc<RwLock<HashMap<String, PositionTracker>>>,
}

impl Default for TrackerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackerManager {
    /// 创建新的追踪器管理器。
    pub fn new() -> Self {
        Self {
            trackers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 返回内部共享状态，便于过渡阶段与遗留代码共享。
    pub fn shared(&self) -> Arc<RwLock<HashMap<String, PositionTracker>>> {
        Arc::clone(&self.trackers)
    }

    /// 同步本地追踪器与交易所真实持仓，避免数量漂移。
    pub async fn sync_trackers(&self, exchange: &impl ExchangeClient) -> Result<()> {
        let positions = exchange.get_positions().await?;
        let mut synced = 0usize;
        let mut removed = 0usize;

        let mut trackers = self.trackers.write().await;
        let mut exchange_symbols: HashSet<String> = HashSet::new();

        for pos in positions.iter() {
            exchange_symbols.insert(pos.symbol.clone());
            if let Some(tracker) = trackers.get_mut(&pos.symbol) {
                let real_qty = pos.size.abs();
                if (tracker.quantity - real_qty).abs() > 0.0001 {
                    warn!(
                        "⚠️  {} tracker 偏差: 本地 {:.8} vs 实际 {:.8}, 已修正",
                        pos.symbol, tracker.quantity, real_qty
                    );
                    tracker.quantity = real_qty;
                    tracker.last_check_time = Utc::now();
                    synced += 1;
                }
            }
        }

        trackers.retain(|symbol, _| {
            let exists = exchange_symbols.contains(symbol);
            if !exists {
                warn!("⚠️  {} 已平仓但 tracker 仍存在,已清理", symbol);
                removed += 1;
            }
            exists
        });

        if synced > 0 || removed > 0 {
            info!("🔄 Tracker 同步完成: 修正 {}, 清理 {}", synced, removed);
        } else {
            debug!("Tracker 同步: 未检测到偏差");
        }

        Ok(())
    }

    /// 清理超过24小时无法确认或无对应持仓的追踪器，防止泄漏。
    pub async fn cleanup_orphaned(&self, exchange: &impl ExchangeClient) -> Result<()> {
        let trackers_snapshot = {
            let trackers = self.trackers.read().await;
            trackers.clone()
        };

        let mut to_remove = Vec::new();

        for (symbol, tracker) in trackers_snapshot.iter() {
            match exchange.get_positions().await {
                Ok(positions) => {
                    let has_position = positions.iter().any(|p| p.symbol == *symbol);
                    if !has_position {
                        info!("🗑️  清理孤立追踪器: {} (无对应持仓)", symbol);
                        to_remove.push(symbol.clone());
                    }
                }
                Err(e) => {
                    warn!("⚠️  获取{}持仓失败(清理检查): {}", symbol, e);
                    warn!("🔍 错误详情: {:?}", e);

                    let age_hours = (Utc::now() - tracker.last_check_time).num_hours();
                    if age_hours >= 24 {
                        warn!("🗑️  清理陈旧追踪器: {} (超过24小时无法验证)", symbol);
                        to_remove.push(symbol.clone());
                    }
                }
            }
        }

        if !to_remove.is_empty() {
            let mut trackers = self.trackers.write().await;
            for symbol in to_remove {
                trackers.remove(&symbol);
            }
        }

        let trackers = self.trackers.read().await;
        if !trackers.is_empty() {
            info!("📊 当前持仓追踪器数: {}", trackers.len());
        }

        Ok(())
    }

    /// 移除指定交易对的追踪器。
    pub fn clear_tracker(&self, symbol: &str) {
        let mut trackers = self.trackers.blocking_write();
        trackers.remove(symbol);
    }

    /// 获取指定交易对的追踪器快照。
    pub fn get_tracker(&self, symbol: &str) -> Option<PositionTracker> {
        let trackers = self.trackers.blocking_read();
        trackers.get(symbol).cloned()
    }

    /// 更新或插入追踪器，供外部在重建时使用。
    pub fn update_tracker(&self, symbol: String, tracker: PositionTracker) {
        let mut trackers = self.trackers.blocking_write();
        trackers.insert(symbol, tracker);
    }
}
