use log::debug;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration as StdDuration, Instant},
};
use tokio::sync::RwLock;

use super::super::modules::{config::VOLATILITY_CACHE_TTL_SECS, types::VolatilityCacheEntry};

/// 波动率缓存管理器
///
/// 通过 `Arc<RwLock<_>>` 提供线程安全的缓存更新、读取与过期清理。
pub struct CacheManager {
    volatility_cache: Arc<RwLock<HashMap<String, VolatilityCacheEntry>>>,
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CacheManager {
    /// 创建新的缓存管理器。
    pub fn new() -> Self {
        Self {
            volatility_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 返回内部缓存句柄，用于共享给其他模块。
    pub fn shared(&self) -> Arc<RwLock<HashMap<String, VolatilityCacheEntry>>> {
        Arc::clone(&self.volatility_cache)
    }

    /// 写入或覆盖波动率缓存。
    pub fn store_volatility(&self, symbol: &str, volatility: f64) {
        let mut cache = self.volatility_cache.blocking_write();
        cache.insert(
            symbol.to_string(),
            VolatilityCacheEntry {
                value: volatility,
                cached_at: Instant::now(),
            },
        );
    }

    /// 获取缓存值，若已过期则自动删除并返回 `None`。
    pub fn get_volatility(&self, symbol: &str) -> Option<f64> {
        let mut cache = self.volatility_cache.blocking_write();
        if let Some(entry) = cache.get(symbol).copied() {
            if entry.cached_at.elapsed() < StdDuration::from_secs(VOLATILITY_CACHE_TTL_SECS) {
                return Some(entry.value);
            }

            cache.remove(symbol);
        }
        None
    }

    /// 主动清理所有过期条目，避免缓存无限增长。
    pub fn cleanup_expired(&self) {
        let mut cache = self.volatility_cache.blocking_write();
        let before = cache.len();
        cache.retain(|_, entry| {
            entry.cached_at.elapsed() < StdDuration::from_secs(VOLATILITY_CACHE_TTL_SECS)
        });

        let removed = before.saturating_sub(cache.len());
        if removed > 0 {
            debug!("🧹 波动率缓存清理完成: 移除 {} 条", removed);
        }
    }
}
