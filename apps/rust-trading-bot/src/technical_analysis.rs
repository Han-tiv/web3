use crate::deepseek_client::{Kline, TechnicalIndicators};
use log::info;

pub struct TechnicalAnalyzer;

impl TechnicalAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// 计算所有技术指标
    pub fn calculate_indicators(&self, klines: &[Kline]) -> TechnicalIndicators {
        let closes: Vec<f64> = klines.iter().map(|k| k.close).collect();

        let sma_5 = self.calculate_sma(&closes, 5);
        let sma_20 = self.calculate_sma(&closes, 20);
        let sma_50 = self.calculate_sma(&closes, 50);
        let rsi = self.calculate_rsi(&closes, 14);
        let (macd, macd_signal) = self.calculate_macd(&closes);
        let (bb_upper, bb_middle, bb_lower) = self.calculate_bollinger_bands(&closes, 20, 2.0);

        info!(
            "📊 技术指标: SMA5={:.2} SMA20={:.2} RSI={:.2}",
            sma_5, sma_20, rsi
        );

        TechnicalIndicators {
            sma_5,
            sma_20,
            sma_50,
            rsi,
            macd,
            macd_signal,
            bb_upper,
            bb_middle,
            bb_lower,
        }
    }

    /// 计算简单移动平均线 (SMA)
    fn calculate_sma(&self, prices: &[f64], period: usize) -> f64 {
        if prices.is_empty() {
            return 0.0;
        }

        if prices.len() < period {
            return prices.iter().sum::<f64>() / prices.len() as f64;
        }

        let sum: f64 = prices.iter().rev().take(period).sum();
        sum / period as f64
    }

    /// 计算指数移动平均线 (EMA)
    fn calculate_ema(&self, prices: &[f64], period: usize) -> f64 {
        if prices.is_empty() || period == 0 {
            return 0.0;
        }

        if prices.len() < period {
            return self.calculate_sma(prices, prices.len());
        }

        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut ema = self.calculate_sma(&prices[..period], period);

        for &price in &prices[period..] {
            ema = (price - ema) * multiplier + ema;
        }

        ema
    }

    /// 计算相对强弱指标 (RSI)
    fn calculate_rsi(&self, prices: &[f64], period: usize) -> f64 {
        if prices.len() < period + 1 {
            return 50.0; // 默认值
        }

        let mut gains = Vec::new();
        let mut losses = Vec::new();

        // 计算价格变化
        for i in 1..prices.len() {
            let change = prices[i] - prices[i - 1];
            if change > 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(change.abs());
            }
        }

        if gains.len() < period {
            return 50.0;
        }

        // 计算平均涨跌幅
        let avg_gain: f64 = gains.iter().rev().take(period).sum::<f64>() / period as f64;
        let avg_loss: f64 = losses.iter().rev().take(period).sum::<f64>() / period as f64;

        if avg_loss == 0.0 {
            return 100.0;
        }

        let rs = avg_gain / avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    }

    /// 计算 MACD
    fn calculate_macd(&self, prices: &[f64]) -> (f64, f64) {
        if prices.len() < 26 {
            return (0.0, 0.0);
        }

        let ema_12 = self.calculate_ema(prices, 12);
        let ema_26 = self.calculate_ema(prices, 26);
        let macd = ema_12 - ema_26;

        // MACD 信号线 (MACD 的 9 日 EMA)
        // 简化版：这里需要 MACD 历史数据，暂时返回简化值
        let macd_signal = macd * 0.9;

        (macd, macd_signal)
    }

    /// 计算布林带
    pub fn calculate_bollinger_bands(
        &self,
        prices: &[f64],
        period: usize,
        std_dev: f64,
    ) -> (f64, f64, f64) {
        if prices.len() < period {
            let avg = prices.iter().sum::<f64>() / prices.len() as f64;
            return (avg, avg, avg);
        }

        let sma = self.calculate_sma(prices, period);
        let variance = self.calculate_variance(prices, period, sma);
        let std = variance.sqrt();

        let upper = sma + (std_dev * std);
        let lower = sma - (std_dev * std);

        (upper, sma, lower)
    }

    /// 计算方差
    fn calculate_variance(&self, prices: &[f64], period: usize, mean: f64) -> f64 {
        if prices.len() < period {
            return 0.0;
        }

        let sum_sq_diff: f64 = prices
            .iter()
            .rev()
            .take(period)
            .map(|&price| (price - mean).powi(2))
            .sum();

        sum_sq_diff / period as f64
    }

    /// 判断趋势
    pub fn determine_trend(&self, indicators: &TechnicalIndicators, current_price: f64) -> String {
        let price_above_sma20 = current_price > indicators.sma_20;
        let price_above_sma50 = current_price > indicators.sma_50;
        let sma20_above_sma50 = indicators.sma_20 > indicators.sma_50;
        let macd_positive = indicators.macd > indicators.macd_signal;

        if price_above_sma20 && price_above_sma50 && sma20_above_sma50 && macd_positive {
            "强势上涨".to_string()
        } else if price_above_sma20 && sma20_above_sma50 {
            "上涨趋势".to_string()
        } else if !price_above_sma20 && !price_above_sma50 && !sma20_above_sma50 && !macd_positive {
            "强势下跌".to_string()
        } else if !price_above_sma20 && !sma20_above_sma50 {
            "下跌趋势".to_string()
        } else {
            "震荡整理".to_string()
        }
    }

    /// 获取超买超卖信号
    pub fn get_rsi_signal(&self, rsi: f64) -> String {
        if rsi > 70.0 {
            "超买 (考虑卖出)".to_string()
        } else if rsi < 30.0 {
            "超卖 (考虑买入)".to_string()
        } else if rsi > 60.0 {
            "偏强".to_string()
        } else if rsi < 40.0 {
            "偏弱".to_string()
        } else {
            "中性".to_string()
        }
    }

    /// 获取布林带信号
    pub fn get_bollinger_signal(
        &self,
        current_price: f64,
        bb_upper: f64,
        bb_lower: f64,
        _bb_middle: f64,
    ) -> String {
        let width = bb_upper - bb_lower;
        let position = (current_price - bb_lower) / width;

        if current_price > bb_upper {
            "突破上轨 (超买)".to_string()
        } else if current_price < bb_lower {
            "突破下轨 (超卖)".to_string()
        } else if position > 0.8 {
            "接近上轨".to_string()
        } else if position < 0.2 {
            "接近下轨".to_string()
        } else {
            "布林带中轨附近".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_calculation() {
        let analyzer = TechnicalAnalyzer::new();
        let prices = vec![100.0, 110.0, 105.0, 115.0, 120.0];
        let sma = analyzer.calculate_sma(&prices, 5);
        assert!((sma - 110.0).abs() < 0.01);
    }

    #[test]
    fn test_rsi_calculation() {
        let analyzer = TechnicalAnalyzer::new();
        let prices = vec![
            44.0, 44.25, 44.5, 43.75, 44.0, 44.5, 45.0, 45.25, 45.5, 45.25, 45.5, 46.0, 45.75,
            45.5, 45.0,
        ];
        let rsi = analyzer.calculate_rsi(&prices, 14);
        assert!(rsi > 0.0 && rsi < 100.0);
    }
}
