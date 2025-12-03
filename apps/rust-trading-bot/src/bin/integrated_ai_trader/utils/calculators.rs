//! 指标计算工具模块
//!
//! 集中实现交易所无关的关键计算逻辑，例如波动率测算。

use anyhow::Result;
use log::{debug, warn};
use rust_trading_bot::{deepseek_client::Kline, exchange_trait::ExchangeClient};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration as StdDuration, Instant},
};
use tokio::{
    sync::RwLock,
    time::{timeout, Duration as TokioDuration},
};

use super::super::{
    VolatilityCacheEntry, DEFAULT_VOLATILITY_PERCENT, VOLATILITY_CACHE_TTL_SECS,
    VOLATILITY_LOOKBACK, VOLATILITY_TIMEOUT_SECS,
};

/// 计算特定交易对的波动率（%），并使用缓存避免重复请求。
///
/// - 优先返回 1 小时内的缓存结果
/// - 超时或数据不足时使用默认值
/// - 依赖任意实现 `ExchangeClient` 的交易所客户端
#[allow(dead_code)]
pub async fn calculate_volatility<E>(
    exchange: &E,
    symbol: &str,
    volatility_cache: &Arc<RwLock<HashMap<String, VolatilityCacheEntry>>>,
) -> Result<f64>
where
    E: ExchangeClient + Sync + Send,
{
    if let Some(entry) = {
        let cache = volatility_cache.read().await;
        cache.get(symbol).copied()
    } {
        if entry.cached_at.elapsed() < StdDuration::from_secs(VOLATILITY_CACHE_TTL_SECS) {
            debug!("📊 波动率缓存命中: {} => {:.2}%", symbol, entry.value);
            return Ok(entry.value);
        }
    }

    let klines_result = timeout(
        TokioDuration::from_secs(VOLATILITY_TIMEOUT_SECS),
        exchange.get_klines(symbol, "15m", Some(VOLATILITY_LOOKBACK)),
    )
    .await;

    let raw_klines = match klines_result {
        Ok(Ok(data)) => data,
        Ok(Err(err)) => {
            warn!(
                "⚠️  获取{} 15m K线计算波动率失败: {}，使用默认值",
                symbol, err
            );
            store_volatility_cache(volatility_cache, symbol, DEFAULT_VOLATILITY_PERCENT).await;
            return Ok(DEFAULT_VOLATILITY_PERCENT);
        }
        Err(_) => {
            warn!(
                "⚠️  获取{} 15m K线计算波动率超时(>{}s)，使用默认值",
                symbol, VOLATILITY_TIMEOUT_SECS
            );
            store_volatility_cache(volatility_cache, symbol, DEFAULT_VOLATILITY_PERCENT).await;
            return Ok(DEFAULT_VOLATILITY_PERCENT);
        }
    };

    let klines: Vec<Kline> = raw_klines
        .into_iter()
        .map(|candle| Kline {
            timestamp: candle.first().copied().unwrap_or_default() as i64,
            open: candle.get(1).copied().unwrap_or_default(),
            high: candle.get(2).copied().unwrap_or_default(),
            low: candle.get(3).copied().unwrap_or_default(),
            close: candle.get(4).copied().unwrap_or_default(),
            volume: candle.get(5).copied().unwrap_or_default(),
            quote_volume: candle.get(6).copied().unwrap_or(0.0),
            taker_buy_volume: candle.get(7).copied().unwrap_or(0.0),
            taker_buy_quote_volume: candle.get(8).copied().unwrap_or(0.0),
        })
        .collect();

    if klines.len() < 2 {
        warn!(
            "⚠️  {} 15m K线数量不足({})，无法计算波动率，使用默认值",
            symbol,
            klines.len()
        );
        store_volatility_cache(volatility_cache, symbol, DEFAULT_VOLATILITY_PERCENT).await;
        return Ok(DEFAULT_VOLATILITY_PERCENT);
    }

    let mut prev_close = klines[0].close;
    let mut tr_total = 0.0;
    let mut samples = 0usize;

    for candle in klines.iter().skip(1) {
        let hl = (candle.high - candle.low).abs();
        let hc = (candle.high - prev_close).abs();
        let lc = (candle.low - prev_close).abs();
        let tr = hl.max(hc).max(lc);
        tr_total += tr;
        samples += 1;
        prev_close = candle.close;
    }

    if samples == 0 {
        warn!("⚠️  {} 触发波动率计算时 TR 样本为空，使用默认值", symbol);
        store_volatility_cache(volatility_cache, symbol, DEFAULT_VOLATILITY_PERCENT).await;
        return Ok(DEFAULT_VOLATILITY_PERCENT);
    }

    let atr = tr_total / samples as f64;
    let current_price = klines
        .last()
        .map(|c| c.close)
        .filter(|price| *price > f64::EPSILON)
        .unwrap_or(0.0);

    if current_price <= f64::EPSILON {
        warn!(
            "⚠️  {} 当前价格异常({:.6})，无法计算波动率，使用默认值",
            symbol, current_price
        );
        store_volatility_cache(volatility_cache, symbol, DEFAULT_VOLATILITY_PERCENT).await;
        return Ok(DEFAULT_VOLATILITY_PERCENT);
    }

    let volatility = ((atr / current_price) * 100.0).max(0.0);
    debug!(
        "📊 {} 波动率计算完成: ATR {:.4}, Price {:.4}, Vol {:.2}%",
        symbol, atr, current_price, volatility
    );

    store_volatility_cache(volatility_cache, symbol, volatility).await;
    Ok(volatility)
}

async fn store_volatility_cache(
    cache: &Arc<RwLock<HashMap<String, VolatilityCacheEntry>>>,
    symbol: &str,
    value: f64,
) {
    let mut writer = cache.write().await;
    writer.insert(
        symbol.to_string(),
        VolatilityCacheEntry {
            value,
            cached_at: Instant::now(),
        },
    );
}
