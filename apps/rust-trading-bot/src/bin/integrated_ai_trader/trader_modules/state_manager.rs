//! State Manager Module
//!
//! 管理IntegratedAITrader的各种状态容器

use chrono::{DateTime, Utc};
use log::info;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use rust_trading_bot::signals::FundAlert;

/// 波动率缓存条目
#[derive(Clone, Debug)]
pub struct VolatilityCacheEntry {
    pub value: f64,
    pub timestamp: DateTime<Utc>,
}

/// 状态管理器
pub struct StateManager {
    /// 追踪的币种
    tracked_coins: Arc<RwLock<HashMap<String, FundAlert>>>,
    /// 波动率缓存
    volatility_cache: Arc<RwLock<HashMap<String, VolatilityCacheEntry>>>,
    /// 上次分析时间
    last_analysis_time: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    /// 配置
    max_tracked_coins: usize,
    coin_ttl_hours: i64,
}

impl StateManager {
    /// 创建新的状态管理器
    pub fn new(
        tracked_coins: Arc<RwLock<HashMap<String, FundAlert>>>,
        volatility_cache: Arc<RwLock<HashMap<String, VolatilityCacheEntry>>>,
        last_analysis_time: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
        max_tracked_coins: usize,
        coin_ttl_hours: i64,
    ) -> Self {
        Self {
            tracked_coins,
            volatility_cache,
            last_analysis_time,
            max_tracked_coins,
            coin_ttl_hours,
        }
    }

    /// 清理过期的追踪币种
    pub async fn cleanup_tracked_coins(&self) {
        let mut coins = self.tracked_coins.write().await;
        let now = Utc::now();
        let ttl_seconds = self.coin_ttl_hours * 3600;

        // 找出所有过期的币种
        let expired: Vec<String> = coins
            .iter()
            .filter(|(_, alert)| {
                let elapsed = now.timestamp() - alert.timestamp;
                elapsed > ttl_seconds
            })
            .map(|(symbol, _)| symbol.clone())
            .collect();

        // 删除过期币种
        for symbol in &expired {
            coins.remove(symbol);
        }

        if !expired.is_empty() {
            info!(
                "🧹 清理了 {} 个过期追踪币种 (TTL: {}小时)",
                expired.len(),
                self.coin_ttl_hours
            );
        }

        // 如果数量仍然超过限制，删除最旧的
        if coins.len() > self.max_tracked_coins {
            let mut sorted: Vec<_> = coins.iter().collect();
            sorted.sort_by_key(|(_, alert)| alert.timestamp);

            let to_remove = coins.len() - self.max_tracked_coins;
            for (symbol, _) in sorted.iter().take(to_remove) {
                coins.remove(*symbol);
            }

            info!(
                "🧹 清理了 {} 个最旧的追踪币种 (限制: {})",
                to_remove, self.max_tracked_coins
            );
        }
    }

    /// 存储波动率缓存
    pub async fn store_volatility_cache(&self, symbol: &str, value: f64) {
        let mut cache = self.volatility_cache.write().await;
        cache.insert(
            symbol.to_string(),
            VolatilityCacheEntry {
                value,
                timestamp: Utc::now(),
            },
        );
    }

    /// 获取波动率缓存
    pub async fn get_volatility_cache(&self, symbol: &str) -> Option<f64> {
        let cache = self.volatility_cache.read().await;
        cache.get(symbol).map(|entry| entry.value)
    }

    /// 更新上次分析时间
    pub async fn update_last_analysis_time(&self, symbol: &str) {
        let mut times = self.last_analysis_time.write().await;
        times.insert(symbol.to_string(), Utc::now());
    }

    /// 获取上次分析时间
    pub async fn get_last_analysis_time(&self, symbol: &str) -> Option<DateTime<Utc>> {
        let times = self.last_analysis_time.read().await;
        times.get(symbol).copied()
    }

    /// 添加追踪币种
    pub async fn add_tracked_coin(&self, symbol: String, alert: FundAlert) {
        let mut coins = self.tracked_coins.write().await;
        coins.insert(symbol, alert);
    }

    /// 获取追踪币种
    pub async fn get_tracked_coin(&self, symbol: &str) -> Option<FundAlert> {
        let coins = self.tracked_coins.read().await;
        coins.get(symbol).cloned()
    }

    /// 获取所有追踪币种
    pub async fn get_all_tracked_coins(&self) -> HashMap<String, FundAlert> {
        let coins = self.tracked_coins.read().await;
        coins.clone()
    }

    /// 获取追踪币种数量
    pub async fn tracked_coins_count(&self) -> usize {
        let coins = self.tracked_coins.read().await;
        coins.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_state_manager_cleanup() {
        let tracked_coins = Arc::new(RwLock::new(HashMap::new()));
        let volatility_cache = Arc::new(RwLock::new(HashMap::new()));
        let last_analysis_time = Arc::new(RwLock::new(HashMap::new()));

        let manager = StateManager::new(
            tracked_coins.clone(),
            volatility_cache,
            last_analysis_time,
            100,
            24,
        );

        // 添加一个旧的追踪币种
        let old_alert = FundAlert {
            coin: "BTCUSDT".to_string(),
            timestamp: Utc::now().timestamp() - 25 * 3600, // 25小时前
            ..Default::default()
        };

        manager
            .add_tracked_coin("BTCUSDT".to_string(), old_alert)
            .await;

        assert_eq!(manager.tracked_coins_count().await, 1);

        // 清理
        manager.cleanup_tracked_coins().await;

        // 应该被删除
        assert_eq!(manager.tracked_coins_count().await, 0);
    }

    #[tokio::test]
    async fn test_volatility_cache() {
        let tracked_coins = Arc::new(RwLock::new(HashMap::new()));
        let volatility_cache = Arc::new(RwLock::new(HashMap::new()));
        let last_analysis_time = Arc::new(RwLock::new(HashMap::new()));

        let manager = StateManager::new(tracked_coins, volatility_cache, last_analysis_time, 100, 24);

        manager.store_volatility_cache("BTCUSDT", 0.5).await;

        let value = manager.get_volatility_cache("BTCUSDT").await;
        assert_eq!(value, Some(0.5));
    }
}
