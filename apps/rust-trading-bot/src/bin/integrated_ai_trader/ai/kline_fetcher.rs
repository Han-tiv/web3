use anyhow::{anyhow, Result};
use log::{info, warn};
use rust_trading_bot::{deepseek_client::Kline, exchange_trait::ExchangeClient};
use std::sync::Arc;
use tokio::time;

pub struct KlineFetcher {
    exchange: Arc<dyn ExchangeClient + Send + Sync>,
}

impl KlineFetcher {
    pub fn new(exchange: Arc<dyn ExchangeClient + Send + Sync>) -> Self {
        Self { exchange }
    }

    /// 并发获取多周期K线数据
    pub async fn fetch_multi_timeframe(
        &self,
        symbol: &str,
    ) -> Result<(Vec<Kline>, Vec<Kline>, Vec<Kline>)> {
        let (klines_5m_result, klines_15m_result, klines_1h_result) = tokio::join!(
            time::timeout(
                time::Duration::from_secs(10),
                self.exchange.get_klines(symbol, "5m", Some(50))
            ),
            time::timeout(
                time::Duration::from_secs(10),
                self.exchange.get_klines(symbol, "15m", Some(50))
            ),
            time::timeout(
                time::Duration::from_secs(10),
                self.exchange.get_klines(symbol, "1h", Some(50))
            )
        );

        // 解析5m K线
        let klines_5m = match klines_5m_result {
            Ok(Ok(data)) => data
                .iter()
                .map(|candle| Kline {
                    timestamp: candle[0] as i64,
                    open: candle[1],
                    high: candle[2],
                    low: candle[3],
                    close: candle[4],
                    volume: candle[5],
                    quote_volume: if candle.len() > 6 { candle[6] } else { 0.0 },
                    taker_buy_volume: if candle.len() > 7 { candle[7] } else { 0.0 },
                    taker_buy_quote_volume: if candle.len() > 8 { candle[8] } else { 0.0 },
                })
                .collect::<Vec<_>>(),
            Ok(Err(e)) => {
                warn!("❌ 获取{}5mK线失败: {}", symbol, e);
                return Err(anyhow!("fetch 5m klines failed"));
            }
            Err(_) => {
                warn!("❌ 获取{}5mK线超时", symbol);
                return Err(anyhow!("fetch 5m klines timeout"));
            }
        };

        // 解析15m K线
        let klines_15m = match klines_15m_result {
            Ok(Ok(data)) => data
                .iter()
                .map(|candle| Kline {
                    timestamp: candle[0] as i64,
                    open: candle[1],
                    high: candle[2],
                    low: candle[3],
                    close: candle[4],
                    volume: candle[5],
                    quote_volume: if candle.len() > 6 { candle[6] } else { 0.0 },
                    taker_buy_volume: if candle.len() > 7 { candle[7] } else { 0.0 },
                    taker_buy_quote_volume: if candle.len() > 8 { candle[8] } else { 0.0 },
                })
                .collect::<Vec<_>>(),
            Ok(Err(e)) => {
                warn!("❌ 获取{}K线失败: {}", symbol, e);
                return Err(anyhow!("fetch 15m klines failed"));
            }
            Err(_) => {
                warn!("❌ 获取{}K线超时", symbol);
                return Err(anyhow!("fetch 15m klines timeout"));
            }
        };

        // 解析1h K线
        let klines_1h = match klines_1h_result {
            Ok(Ok(data)) => data
                .iter()
                .map(|candle| Kline {
                    timestamp: candle[0] as i64,
                    open: candle[1],
                    high: candle[2],
                    low: candle[3],
                    close: candle[4],
                    volume: candle[5],
                    quote_volume: if candle.len() > 6 { candle[6] } else { 0.0 },
                    taker_buy_volume: if candle.len() > 7 { candle[7] } else { 0.0 },
                    taker_buy_quote_volume: if candle.len() > 8 { candle[8] } else { 0.0 },
                })
                .collect::<Vec<_>>(),
            Ok(Err(e)) => {
                warn!("❌ 获取{}1hK线失败: {}", symbol, e);
                return Err(anyhow!("fetch 1h klines failed"));
            }
            Err(_) => {
                warn!("❌ 获取{}1hK线超时", symbol);
                return Err(anyhow!("fetch 1h klines timeout"));
            }
        };

        if klines_1h.len() < 20 {
            warn!("⚠️  1h K线数据不足: {} (需要至少20根)", klines_1h.len());
            return Err(anyhow!("not enough 1h klines"));
        }

        if let Some(last_hour) = klines_1h.last() {
            info!(
                "🕒 1h 最新K线: 收盘价 ${:.4} | 成交量 {:.2}",
                last_hour.close, last_hour.volume
            );
        }

        if klines_15m.len() < 20 {
            warn!("⚠️  K线数据不足: {} (需要至少20根)", klines_15m.len());
            return Err(anyhow!("not enough 15m klines"));
        }

        Ok((klines_5m, klines_15m, klines_1h))
    }
}
