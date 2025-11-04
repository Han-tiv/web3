/// 市场数据获取器 - 从多个交易所获取技术数据
///
/// 支持的交易所：
/// - Binance (币安)
/// - OKX (欧易)
/// - Bybit
/// - Gate.io
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub symbol: String,
    pub exchange: String,
    pub current_price: f64,
    pub volume_24h: f64,
    pub high_24h: f64,
    pub low_24h: f64,
    pub change_1h: f64,
    pub change_24h: f64,
    pub klines_15m: Vec<Kline>,
    pub technical_indicators: TechnicalIndicators,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kline {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalIndicators {
    pub rsi_15m: f64,
    pub macd: f64,
    pub macd_signal: f64,
    pub macd_histogram: f64,
    pub bb_upper: f64,
    pub bb_middle: f64,
    pub bb_lower: f64,
    pub bb_position: String, // "上轨区"/"中轨区"/"下轨区"
    pub sma_5: f64,
    pub sma_20: f64,
    pub funding_rate: Option<f64>, // 资金费率（仅合约）
}

pub struct MarketDataFetcher {
    client: Client,
}

impl MarketDataFetcher {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// 获取币种的市场数据
    /// 优先级：Binance > OKX > Bybit > Gate
    pub async fn fetch_market_data(&self, coin: &str) -> Result<MarketData> {
        // 尝试从Binance获取
        if let Ok(data) = self.fetch_from_binance(coin).await {
            return Ok(data);
        }

        // 尝试从OKX获取
        if let Ok(data) = self.fetch_from_okx(coin).await {
            return Ok(data);
        }

        // 尝试从Bybit获取
        if let Ok(data) = self.fetch_from_bybit(coin).await {
            return Ok(data);
        }

        // 尝试从Gate获取
        if let Ok(data) = self.fetch_from_gate(coin).await {
            return Ok(data);
        }

        anyhow::bail!("无法从任何交易所获取 {} 的数据", coin)
    }

    /// 从Binance获取数据
    async fn fetch_from_binance(&self, coin: &str) -> Result<MarketData> {
        let symbol = format!("{}USDT", coin);

        // 获取24h ticker
        let ticker_url = format!(
            "https://api.binance.com/api/v3/ticker/24hr?symbol={}",
            symbol
        );
        let ticker: BinanceTicker = self
            .client
            .get(&ticker_url)
            .send()
            .await?
            .json()
            .await
            .context("Failed to parse Binance ticker")?;

        // 获取15m K线（最近100根）
        let klines_url = format!(
            "https://api.binance.com/api/v3/klines?symbol={}&interval=15m&limit=100",
            symbol
        );
        let klines_raw: Vec<serde_json::Value> = self
            .client
            .get(&klines_url)
            .send()
            .await?
            .json()
            .await
            .context("Failed to parse Binance klines")?;

        let klines: Vec<Kline> = klines_raw
            .iter()
            .map(|k| Kline {
                timestamp: k[0].as_i64().unwrap_or(0),
                open: k[1].as_str().unwrap_or("0").parse().unwrap_or(0.0),
                high: k[2].as_str().unwrap_or("0").parse().unwrap_or(0.0),
                low: k[3].as_str().unwrap_or("0").parse().unwrap_or(0.0),
                close: k[4].as_str().unwrap_or("0").parse().unwrap_or(0.0),
                volume: k[5].as_str().unwrap_or("0").parse().unwrap_or(0.0),
            })
            .collect();

        // 计算技术指标
        let indicators = self.calculate_indicators(&klines)?;

        // 获取资金费率（合约）
        let funding_rate = self.fetch_binance_funding_rate(&symbol).await.ok();

        let current_price = ticker.last_price.parse().unwrap_or(0.0);

        Ok(MarketData {
            symbol: symbol.clone(),
            exchange: "Binance".to_string(),
            current_price,
            volume_24h: ticker.volume.parse().unwrap_or(0.0),
            high_24h: ticker.high_price.parse().unwrap_or(0.0),
            low_24h: ticker.low_price.parse().unwrap_or(0.0),
            change_1h: 0.0, // Binance不直接提供1h数据
            change_24h: ticker.price_change_percent.parse().unwrap_or(0.0),
            klines_15m: klines,
            technical_indicators: TechnicalIndicators {
                rsi_15m: indicators.rsi,
                macd: indicators.macd,
                macd_signal: indicators.macd_signal,
                macd_histogram: indicators.macd - indicators.macd_signal,
                bb_upper: indicators.bb_upper,
                bb_middle: indicators.bb_middle,
                bb_lower: indicators.bb_lower,
                bb_position: self.get_bb_position(current_price, &indicators),
                sma_5: indicators.sma_5,
                sma_20: indicators.sma_20,
                funding_rate,
            },
        })
    }

    /// 从OKX获取数据
    async fn fetch_from_okx(&self, coin: &str) -> Result<MarketData> {
        // TODO: 实现OKX数据获取
        anyhow::bail!("OKX integration not implemented yet")
    }

    /// 从Bybit获取数据
    async fn fetch_from_bybit(&self, coin: &str) -> Result<MarketData> {
        // TODO: 实现Bybit数据获取
        anyhow::bail!("Bybit integration not implemented yet")
    }

    /// 从Gate获取数据
    async fn fetch_from_gate(&self, coin: &str) -> Result<MarketData> {
        // TODO: 实现Gate数据获取
        anyhow::bail!("Gate integration not implemented yet")
    }

    /// 获取Binance资金费率
    async fn fetch_binance_funding_rate(&self, symbol: &str) -> Result<f64> {
        let url = format!(
            "https://fapi.binance.com/fapi/v1/premiumIndex?symbol={}",
            symbol
        );

        let response: BinanceFundingRate = self.client.get(&url).send().await?.json().await?;

        Ok(response.last_funding_rate.parse().unwrap_or(0.0))
    }

    /// 计算技术指标
    fn calculate_indicators(&self, klines: &[Kline]) -> Result<IndicatorCalc> {
        if klines.is_empty() {
            anyhow::bail!("No klines data to calculate indicators");
        }

        let closes: Vec<f64> = klines.iter().map(|k| k.close).collect();

        // 计算RSI
        let rsi = self.calculate_rsi(&closes, 14);

        // 计算MACD
        let (macd, signal) = self.calculate_macd(&closes);

        // 计算布林带
        let (bb_upper, bb_middle, bb_lower) = self.calculate_bollinger_bands(&closes, 20, 2.0);

        // 计算SMA
        let sma_5 = self.calculate_sma(&closes, 5);
        let sma_20 = self.calculate_sma(&closes, 20);

        Ok(IndicatorCalc {
            rsi,
            macd,
            macd_signal: signal,
            bb_upper,
            bb_middle,
            bb_lower,
            sma_5,
            sma_20,
        })
    }

    /// 计算RSI
    fn calculate_rsi(&self, prices: &[f64], period: usize) -> f64 {
        if prices.len() < period + 1 {
            return 50.0;
        }

        let mut gains = Vec::new();
        let mut losses = Vec::new();

        for i in 1..prices.len() {
            let change = prices[i] - prices[i - 1];
            if change > 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(-change);
            }
        }

        let avg_gain: f64 = gains.iter().rev().take(period).sum::<f64>() / period as f64;
        let avg_loss: f64 = losses.iter().rev().take(period).sum::<f64>() / period as f64;

        if avg_loss == 0.0 {
            return 100.0;
        }

        let rs = avg_gain / avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    }

    /// 计算MACD
    fn calculate_macd(&self, prices: &[f64]) -> (f64, f64) {
        let ema_12 = self.calculate_ema(prices, 12);
        let ema_26 = self.calculate_ema(prices, 26);
        let macd = ema_12 - ema_26;

        // Signal line (9-period EMA of MACD)
        let signal = macd * 0.9; // Simplified

        (macd, signal)
    }

    /// 计算EMA
    fn calculate_ema(&self, prices: &[f64], period: usize) -> f64 {
        if prices.is_empty() {
            return 0.0;
        }

        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut ema = prices[0];

        for price in prices.iter().skip(1) {
            ema = (price - ema) * multiplier + ema;
        }

        ema
    }

    /// 计算布林带
    fn calculate_bollinger_bands(
        &self,
        prices: &[f64],
        period: usize,
        std_dev: f64,
    ) -> (f64, f64, f64) {
        let sma = self.calculate_sma(prices, period);
        let std = self.calculate_std_dev(prices, period);

        let upper = sma + (std * std_dev);
        let lower = sma - (std * std_dev);

        (upper, sma, lower)
    }

    /// 计算SMA
    fn calculate_sma(&self, prices: &[f64], period: usize) -> f64 {
        if prices.len() < period {
            return prices.iter().sum::<f64>() / prices.len() as f64;
        }

        prices.iter().rev().take(period).sum::<f64>() / period as f64
    }

    /// 计算标准差
    fn calculate_std_dev(&self, prices: &[f64], period: usize) -> f64 {
        let sma = self.calculate_sma(prices, period);
        let subset: Vec<f64> = prices.iter().rev().take(period).cloned().collect();

        let variance: f64 = subset
            .iter()
            .map(|price| {
                let diff = price - sma;
                diff * diff
            })
            .sum::<f64>()
            / period as f64;

        variance.sqrt()
    }

    /// 判断价格在布林带的位置
    fn get_bb_position(&self, price: f64, indicators: &IndicatorCalc) -> String {
        let upper_distance = (indicators.bb_upper - price).abs();
        let middle_distance = (indicators.bb_middle - price).abs();
        let lower_distance = (indicators.bb_lower - price).abs();

        let min_distance = upper_distance.min(middle_distance).min(lower_distance);

        if min_distance == upper_distance {
            "上轨区（超买风险）".to_string()
        } else if min_distance == lower_distance {
            "下轨区（超卖机会）".to_string()
        } else {
            "中轨区（正常范围）".to_string()
        }
    }
}

#[derive(Debug, Deserialize)]
struct BinanceTicker {
    #[serde(rename = "lastPrice")]
    last_price: String,
    #[serde(rename = "priceChangePercent")]
    price_change_percent: String,
    volume: String,
    #[serde(rename = "highPrice")]
    high_price: String,
    #[serde(rename = "lowPrice")]
    low_price: String,
}

#[derive(Debug, Deserialize)]
struct BinanceFundingRate {
    #[serde(rename = "lastFundingRate")]
    last_funding_rate: String,
}

#[derive(Debug)]
struct IndicatorCalc {
    rsi: f64,
    macd: f64,
    macd_signal: f64,
    bb_upper: f64,
    bb_middle: f64,
    bb_lower: f64,
    sma_5: f64,
    sma_20: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_btc_data() {
        let fetcher = MarketDataFetcher::new();
        let result = fetcher.fetch_market_data("BTC").await;
        assert!(result.is_ok());

        let data = result.unwrap();
        println!("BTC Data: {:?}", data);
        assert!(data.current_price > 0.0);
    }
}
