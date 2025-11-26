use anyhow::{Context, Result};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

use crate::deepseek_client::{DeepSeekClient, Kline, TechnicalIndicators, TradingSignal};
use crate::exchange_trait::{ExchangeClient, Position};
use crate::technical_analysis::TechnicalAnalyzer;

/// AI 决策配置
#[derive(Debug, Clone)]
pub struct AiDecisionConfig {
    /// 最大并发AI调用数
    pub max_concurrent_calls: usize,
    /// 单次AI调用超时时间（秒）
    pub call_timeout_secs: u64,
    /// K线获取超时时间（秒）
    pub kline_timeout_secs: u64,
    /// 失败重试次数
    pub max_retries: usize,
    /// K线缓存时间（秒）
    pub kline_cache_ttl_secs: u64,
}

impl Default for AiDecisionConfig {
    fn default() -> Self {
        Self {
            max_concurrent_calls: 5,
            call_timeout_secs: 10,
            kline_timeout_secs: 5,
            max_retries: 3,
            kline_cache_ttl_secs: 900, // 15分钟
        }
    }
}

/// 币种信息（用于批量分析）
#[derive(Debug, Clone)]
pub struct CoinInfo {
    pub symbol: String,
    pub current_position: Option<Position>,
}

/// AI 决策结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiDecision {
    pub symbol: String,
    pub signal: TradingSignal,
    pub indicators: TechnicalIndicators,
    pub current_price: f64,
    pub analysis_timestamp: i64,
}

/// K线缓存项
struct KlineCache {
    klines: Vec<Kline>,
    timestamp: std::time::Instant,
}

/// AI 决策引擎
pub struct AiDecisionEngine {
    config: AiDecisionConfig,
    deepseek: Arc<DeepSeekClient>,
    analyzer: Arc<TechnicalAnalyzer>,
    semaphore: Arc<Semaphore>,
    kline_cache: Arc<tokio::sync::Mutex<HashMap<String, KlineCache>>>,
}

impl AiDecisionEngine {
    pub fn new(
        config: AiDecisionConfig,
        deepseek: Arc<DeepSeekClient>,
        analyzer: Arc<TechnicalAnalyzer>,
    ) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_calls));

        Self {
            config,
            deepseek,
            analyzer,
            semaphore,
            kline_cache: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    /// 批量分析多个币种（并发执行）
    pub async fn analyze_batch(
        &self,
        coins: Vec<CoinInfo>,
        exchange: Arc<dyn ExchangeClient>,
    ) -> Vec<AiDecision> {
        info!("开始批量AI分析: {} 个币种", coins.len());

        let mut tasks = Vec::new();

        for coin_info in coins {
            let engine = self.clone_for_task();
            let exchange_clone = exchange.clone();

            let task =
                tokio::spawn(async move { engine.analyze_single(coin_info, exchange_clone).await });

            tasks.push(task);
        }

        // 等待所有任务完成
        let mut results = Vec::new();
        let total = tasks.len();
        for task in tasks {
            match task.await {
                Ok(Ok(decision)) => results.push(decision),
                Ok(Err(e)) => error!("AI分析失败: {}", e),
                Err(e) => error!("任务执行失败: {}", e),
            }
        }

        info!("批量AI分析完成: {}/{} 成功", results.len(), total);

        results
    }

    /// 分析单个币种
    async fn analyze_single(
        &self,
        coin_info: CoinInfo,
        exchange: Arc<dyn ExchangeClient>,
    ) -> Result<AiDecision> {
        // 获取信号量（限制并发）
        let _permit = self.semaphore.acquire().await?;

        info!("分析币种: {}", coin_info.symbol);

        // 重试逻辑
        let mut last_error = None;
        for attempt in 1..=self.config.max_retries {
            match self.analyze_with_timeout(&coin_info, &exchange).await {
                Ok(decision) => {
                    info!(
                        "✅ {} 分析完成 (尝试 {}/{})",
                        coin_info.symbol, attempt, self.config.max_retries
                    );
                    return Ok(decision);
                }
                Err(e) => {
                    warn!(
                        "⚠️  {} 分析失败 (尝试 {}/{}): {}",
                        coin_info.symbol, attempt, self.config.max_retries, e
                    );
                    last_error = Some(e);

                    if attempt < self.config.max_retries {
                        // 指数退避
                        let backoff = Duration::from_millis(100 * (2_u64.pow(attempt as u32)));
                        tokio::time::sleep(backoff).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("AI分析失败，无错误信息")))
    }

    /// 带超时的分析
    async fn analyze_with_timeout(
        &self,
        coin_info: &CoinInfo,
        exchange: &Arc<dyn ExchangeClient>,
    ) -> Result<AiDecision> {
        let timeout = Duration::from_secs(self.config.call_timeout_secs);

        tokio::time::timeout(timeout, self.analyze_core(coin_info, exchange))
            .await
            .context("AI分析超时")?
    }

    /// 核心分析逻辑
    async fn analyze_core(
        &self,
        coin_info: &CoinInfo,
        exchange: &Arc<dyn ExchangeClient>,
    ) -> Result<AiDecision> {
        // 1. 获取K线数据（带缓存）
        let klines = self.get_klines_cached(&coin_info.symbol, exchange).await?;

        if klines.len() < 50 {
            anyhow::bail!("K线数据不足: {} 根 (需要至少50根)", klines.len());
        }

        let current_price = klines.last().unwrap().close;

        // 2. 计算技术指标
        let indicators = self.analyzer.calculate_indicators(&klines);

        // 3. 构建AI prompt
        let position_ref =
            coin_info
                .current_position
                .as_ref()
                .map(|p| crate::deepseek_client::Position {
                    side: p.side.clone(),
                    size: p.size,
                    entry_price: p.entry_price,
                    unrealized_pnl: p.pnl,
                });

        let prompt =
            self.deepseek
                .build_prompt(&klines, &indicators, current_price, position_ref.as_ref());

        // 4. 调用DeepSeek AI
        let signal = self.deepseek.analyze_market(&prompt).await?;

        Ok(AiDecision {
            symbol: coin_info.symbol.clone(),
            signal,
            indicators,
            current_price,
            analysis_timestamp: chrono::Utc::now().timestamp(),
        })
    }

    /// 获取K线数据（带缓存）
    async fn get_klines_cached(
        &self,
        symbol: &str,
        exchange: &Arc<dyn ExchangeClient>,
    ) -> Result<Vec<Kline>> {
        let cache = self.kline_cache.lock().await;

        // 检查缓存
        if let Some(cached) = cache.get(symbol) {
            let elapsed = cached.timestamp.elapsed();
            if elapsed.as_secs() < self.config.kline_cache_ttl_secs {
                return Ok(cached.klines.clone());
            }
        }

        // 获取新数据
        drop(cache); // 释放锁
        let klines = self.fetch_klines_with_timeout(symbol, exchange).await?;

        // 更新缓存
        let mut cache = self.kline_cache.lock().await;
        cache.insert(
            symbol.to_string(),
            KlineCache {
                klines: klines.clone(),
                timestamp: std::time::Instant::now(),
            },
        );

        Ok(klines)
    }

    /// 带超时的K线获取
    async fn fetch_klines_with_timeout(
        &self,
        symbol: &str,
        exchange: &Arc<dyn ExchangeClient>,
    ) -> Result<Vec<Kline>> {
        let timeout = Duration::from_secs(self.config.kline_timeout_secs);

        tokio::time::timeout(timeout, self.fetch_klines(symbol, exchange))
            .await
            .context("获取K线超时")?
    }

    /// 实际获取K线数据
    async fn fetch_klines(
        &self,
        symbol: &str,
        exchange: &Arc<dyn ExchangeClient>,
    ) -> Result<Vec<Kline>> {
        // 获取最近 100 根 1 小时 K 线
        let raw_klines = exchange.get_klines(symbol, "1h", Some(100)).await?;

        // 转换为 Kline 结构
        let klines: Vec<Kline> = raw_klines
            .iter()
            .map(|k| Kline {
                timestamp: k[0] as i64,
                open: k[1],
                high: k[2],
                low: k[3],
                close: k[4],
                volume: k[5],
                quote_volume: if k.len() > 6 { k[6] } else { 0.0 },
                taker_buy_volume: if k.len() > 7 { k[7] } else { 0.0 },
                taker_buy_quote_volume: if k.len() > 8 { k[8] } else { 0.0 },
            })
            .collect();

        Ok(klines)
    }

    /// 清理过期缓存
    pub async fn cleanup_cache(&self) {
        let mut cache = self.kline_cache.lock().await;
        let before_count = cache.len();

        cache.retain(|_, item| {
            item.timestamp.elapsed().as_secs() < self.config.kline_cache_ttl_secs
        });

        let removed = before_count - cache.len();
        if removed > 0 {
            info!("清理K线缓存: 移除 {} 条过期记录", removed);
        }
    }

    /// 启动后台缓存清理任务
    pub fn start_cache_cleanup_task(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                Duration::from_secs(300), // 每5分钟清理一次
            );

            loop {
                interval.tick().await;
                self.cleanup_cache().await;
            }
        });
    }

    /// 克隆用于异步任务
    fn clone_for_task(&self) -> Self {
        Self {
            config: self.config.clone(),
            deepseek: self.deepseek.clone(),
            analyzer: self.analyzer.clone(),
            semaphore: self.semaphore.clone(),
            kline_cache: self.kline_cache.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 注意：这些测试需要模拟的 exchange 和 deepseek 客户端
    // 在实际项目中应该使用 mock 框架

    #[tokio::test]
    async fn test_concurrent_limit() {
        // 测试并发限制是否生效
        // 需要 mock 实现
    }
}
