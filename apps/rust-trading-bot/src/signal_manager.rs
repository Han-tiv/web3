use chrono::{DateTime, Duration, Utc};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::Mutex;

/// 信号来源
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignalSource {
    Channel { id: i64, name: String },
    Position,
    Manual,
}

/// 币种信号优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SignalPriority {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

/// 币种信号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinSignal {
    pub symbol: String,                    // 币种符号 (如 "BTCUSDT")
    pub source: SignalSource,              // 信号来源
    pub priority: SignalPriority,          // 优先级
    pub timestamp: DateTime<Utc>,          // 时间戳
    pub raw_data: Option<String>,          // 原始数据（频道消息等）
    pub metadata: HashMap<String, String>, // 额外元数据
}

impl CoinSignal {
    pub fn new(symbol: String, source: SignalSource) -> Self {
        Self {
            symbol,
            source,
            priority: SignalPriority::Medium,
            timestamp: Utc::now(),
            raw_data: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_priority(mut self, priority: SignalPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_raw_data(mut self, raw_data: String) -> Self {
        self.raw_data = Some(raw_data);
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// 生成去重键（symbol + 时间桶）
    pub fn dedup_key(&self, bucket_seconds: i64) -> String {
        let bucket = self.timestamp.timestamp() / bucket_seconds;
        format!("{}:{}", self.symbol, bucket)
    }
}

/// 信号管理器配置
#[derive(Debug, Clone)]
pub struct SignalManagerConfig {
    /// 最大队列长度
    pub max_queue_size: usize,
    /// 去重时间窗口（秒）
    pub dedup_window_secs: i64,
    /// 信号过期时间（秒）
    pub signal_ttl_secs: i64,
}

impl Default for SignalManagerConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 1000,
            dedup_window_secs: 180, // 3分钟
            signal_ttl_secs: 600,   // 10分钟
        }
    }
}

/// 信号管理器统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalStats {
    pub total_received: usize,
    pub total_deduplicated: usize,
    pub total_expired: usize,
    pub current_queue_size: usize,
    pub signals_by_source: HashMap<String, usize>,
}

/// 信号管理器
pub struct SignalManager {
    config: SignalManagerConfig,
    queue: Arc<Mutex<VecDeque<CoinSignal>>>,
    dedup_cache: Arc<Mutex<HashMap<String, DateTime<Utc>>>>,
    stats: Arc<Mutex<SignalStats>>,
}

impl SignalManager {
    pub fn new(config: SignalManagerConfig) -> Self {
        Self {
            config,
            queue: Arc::new(Mutex::new(VecDeque::new())),
            dedup_cache: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(SignalStats {
                total_received: 0,
                total_deduplicated: 0,
                total_expired: 0,
                current_queue_size: 0,
                signals_by_source: HashMap::new(),
            })),
        }
    }

    /// 添加信号到队列
    pub async fn add_signal(&self, signal: CoinSignal) -> bool {
        let dedup_key = signal.dedup_key(self.config.dedup_window_secs);

        // 检查去重
        {
            let mut cache = self.dedup_cache.lock().await;
            if let Some(last_seen) = cache.get(&dedup_key) {
                let elapsed = Utc::now().signed_duration_since(*last_seen);
                if elapsed.num_seconds() < self.config.dedup_window_secs {
                    debug!(
                        "信号去重: {} ({}秒前已处理)",
                        signal.symbol,
                        elapsed.num_seconds()
                    );

                    let mut stats = self.stats.lock().await;
                    stats.total_deduplicated += 1;
                    return false;
                }
            }
            cache.insert(dedup_key, Utc::now());
        }

        // 添加到队列
        {
            let mut queue = self.queue.lock().await;

            // 检查队列大小
            if queue.len() >= self.config.max_queue_size {
                warn!(
                    "信号队列已满 ({}), 丢弃最旧信号",
                    self.config.max_queue_size
                );
                queue.pop_front();
            }

            info!(
                "添加信号: {} | 来源: {:?} | 优先级: {:?}",
                signal.symbol, signal.source, signal.priority
            );

            queue.push_back(signal.clone());
        }

        // 更新统计
        {
            let mut stats = self.stats.lock().await;
            stats.total_received += 1;
            stats.current_queue_size += 1;

            let source_key = match &signal.source {
                SignalSource::Channel { name, .. } => format!("channel:{}", name),
                SignalSource::Position => "position".to_string(),
                SignalSource::Manual => "manual".to_string(),
            };
            *stats.signals_by_source.entry(source_key).or_insert(0) += 1;
        }

        true
    }

    /// 批量添加信号
    pub async fn add_signals(&self, signals: Vec<CoinSignal>) -> usize {
        let mut added = 0;
        for signal in signals {
            if self.add_signal(signal).await {
                added += 1;
            }
        }
        added
    }

    /// 获取最近 N 秒内的信号（清空队列）
    pub async fn drain_recent(&self, seconds: i64) -> Vec<CoinSignal> {
        let cutoff_time = Utc::now() - Duration::seconds(seconds);
        let mut queue = self.queue.lock().await;

        let mut recent_signals = Vec::new();
        let mut expired = 0;

        // 从队列中取出所有符合时间窗口的信号
        while let Some(signal) = queue.pop_front() {
            if signal.timestamp >= cutoff_time {
                recent_signals.push(signal);
            } else {
                expired += 1;
            }
        }

        // 按优先级排序
        recent_signals.sort_by(|a, b| b.priority.cmp(&a.priority));

        // 更新统计
        if expired > 0 || !recent_signals.is_empty() {
            let mut stats = self.stats.lock().await;
            stats.total_expired += expired;
            stats.current_queue_size = queue.len();
        }

        info!(
            "拉取 {} 秒内信号: {} 个, 过期: {} 个",
            seconds,
            recent_signals.len(),
            expired
        );

        recent_signals
    }

    /// 查看队列内容（不清空）
    pub async fn peek_recent(&self, seconds: i64) -> Vec<CoinSignal> {
        let cutoff_time = Utc::now() - Duration::seconds(seconds);
        let queue = self.queue.lock().await;

        queue
            .iter()
            .filter(|s| s.timestamp >= cutoff_time)
            .cloned()
            .collect()
    }

    /// 清理过期的去重缓存
    pub async fn cleanup_cache(&self) {
        let cutoff_time = Utc::now() - Duration::seconds(self.config.dedup_window_secs);
        let mut cache = self.dedup_cache.lock().await;

        let before_count = cache.len();
        cache.retain(|_, &mut timestamp| timestamp >= cutoff_time);
        let removed = before_count - cache.len();

        if removed > 0 {
            debug!("清理去重缓存: 移除 {} 条过期记录", removed);
        }
    }

    /// 获取当前队列大小
    pub async fn queue_size(&self) -> usize {
        self.queue.lock().await.len()
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> SignalStats {
        let stats = self.stats.lock().await;
        let queue_size = self.queue.lock().await.len();

        let mut result = stats.clone();
        result.current_queue_size = queue_size;
        result
    }

    /// 重置统计信息
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.lock().await;
        *stats = SignalStats {
            total_received: 0,
            total_deduplicated: 0,
            total_expired: 0,
            current_queue_size: self.queue.lock().await.len(),
            signals_by_source: HashMap::new(),
        };
    }

    /// 启动后台清理任务
    pub fn start_cleanup_task(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

            loop {
                interval.tick().await;
                self.cleanup_cache().await;
            }
        });
    }

    /// 去重并返回唯一币种列表
    pub fn dedup_symbols(signals: &[CoinSignal]) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut unique_symbols = Vec::new();

        for signal in signals {
            if seen.insert(signal.symbol.clone()) {
                unique_symbols.push(signal.symbol.clone());
            }
        }

        unique_symbols
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_signal_deduplication() {
        let manager = SignalManager::new(SignalManagerConfig {
            dedup_window_secs: 5,
            ..Default::default()
        });

        let signal1 = CoinSignal::new(
            "BTCUSDT".to_string(),
            SignalSource::Channel {
                id: 123,
                name: "test".to_string(),
            },
        );

        // 第一次添加应该成功
        assert!(manager.add_signal(signal1.clone()).await);

        // 5秒内重复添加应该被去重
        assert!(!manager.add_signal(signal1.clone()).await);

        // 等待6秒后应该可以再次添加
        tokio::time::sleep(tokio::time::Duration::from_secs(6)).await;
        assert!(manager.add_signal(signal1).await);
    }

    #[tokio::test]
    async fn test_drain_recent() {
        let manager = SignalManager::new(SignalManagerConfig::default());

        let signal1 = CoinSignal::new("BTCUSDT".to_string(), SignalSource::Position);
        let signal2 = CoinSignal::new("ETHUSDT".to_string(), SignalSource::Position);

        manager.add_signal(signal1).await;
        manager.add_signal(signal2).await;

        let signals = manager.drain_recent(60).await;
        assert_eq!(signals.len(), 2);

        // 再次拉取应该为空
        let signals = manager.drain_recent(60).await;
        assert_eq!(signals.len(), 0);
    }

    #[tokio::test]
    async fn test_priority_sorting() {
        let manager = SignalManager::new(SignalManagerConfig::default());

        let low = CoinSignal::new("BTC".to_string(), SignalSource::Position)
            .with_priority(SignalPriority::Low);
        let high = CoinSignal::new("ETH".to_string(), SignalSource::Position)
            .with_priority(SignalPriority::High);

        manager.add_signal(low).await;
        manager.add_signal(high).await;

        let signals = manager.drain_recent(60).await;

        // 高优先级应该排在前面
        assert_eq!(signals[0].symbol, "ETH");
        assert_eq!(signals[1].symbol, "BTC");
    }
}
