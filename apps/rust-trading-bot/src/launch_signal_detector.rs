/// 启动信号检测模块 - 多周期综合判断
///
/// 核心功能:
/// 1. 5m启动信号检测(连续阳线+成交量放大)
/// 2. 15m趋势确认(SMA20+连续阳线)
/// 3. 1h压力位突破判断
/// 4. 1m实时偏离度计算
/// 5. 综合判断是否满足加仓条件
use crate::deepseek_client::Kline;
use anyhow::Result;
use log::info;

/// 启动信号检测结果
#[derive(Debug, Clone)]
pub struct LaunchSignal {
    pub m5_signal: bool,     // 5m启动信号
    pub m15_trend: bool,     // 15m趋势确认
    pub h1_breakout: bool,   // 1h压力位突破
    pub m1_deviation: f64,   // 1m实时偏离度(%)
    pub m1_strong: bool,     // 1m偏离度是否>0.5%
    pub all_confirmed: bool, // 全部确认
    pub score: f64,          // 综合得分(0-100)
    pub reason: String,      // 详细说明
}

/// 启动信号检测器
pub struct LaunchSignalDetector {
    // 5m启动信号参数
    pub m5_consecutive_bullish: usize, // 3 (连续阳线数)
    pub m5_body_min_pct: f64,          // 0.5% (最小实体%)
    pub m5_volume_increase: f64,       // 0.3 (成交量放大30%)

    // 15m趋势确认参数
    pub m15_sma_period: usize,          // 20 (SMA周期)
    pub m15_consecutive_bullish: usize, // 2 (连续阳线数)

    // 1h突破参数
    pub h1_breakout_min_pct: f64, // 1.5% (最小突破幅度)
    pub h1_lookback: usize,       // 5 (分析最近5根1h)

    // 1m偏离度参数
    pub m1_strong_threshold: f64, // 0.5% (强势阈值)
}

impl Default for LaunchSignalDetector {
    fn default() -> Self {
        Self {
            m5_consecutive_bullish: 3,
            m5_body_min_pct: 0.5,
            m5_volume_increase: 0.3,

            m15_sma_period: 20,
            m15_consecutive_bullish: 2,

            h1_breakout_min_pct: 1.5,
            h1_lookback: 5,

            m1_strong_threshold: 0.5,
        }
    }
}

impl LaunchSignalDetector {
    /// 综合检测启动信号
    pub fn detect_launch_signal(
        &self,
        klines_5m: &[Kline],
        klines_15m: &[Kline],
        klines_1h: &[Kline],
        entry_price: f64,
        current_price: f64,
    ) -> Result<LaunchSignal> {
        // 1. 检测5m启动信号
        let m5_signal = self.detect_5m_launch(klines_5m)?;

        // 2. 检测15m趋势确认
        let m15_trend = self.confirm_15m_trend(klines_15m)?;

        // 3. 检测1h压力位突破
        let h1_breakout = self.check_1h_breakout(klines_1h, entry_price, current_price)?;

        // 4. 计算1m实时偏离度
        let m1_deviation = self.calculate_1m_deviation(klines_5m, current_price);
        let m1_strong = m1_deviation > self.m1_strong_threshold;

        // 5. 综合判断
        let all_confirmed = m5_signal && m15_trend && h1_breakout && m1_strong;

        // 6. 计算得分
        let mut score = 0.0;
        if m5_signal {
            score += 40.0;
        }
        if m15_trend {
            score += 30.0;
        }
        if h1_breakout {
            score += 20.0;
        }
        if m1_strong {
            score += 10.0;
        }

        // 7. 生成说明
        let reason = format!(
            "5m启动:{} | 15m趋势:{} | 1h突破:{} | 1m偏离:{:+.2}% | 得分:{:.0}/100",
            if m5_signal { "✅" } else { "❌" },
            if m15_trend { "✅" } else { "❌" },
            if h1_breakout { "✅" } else { "❌" },
            m1_deviation,
            score
        );

        info!("🚀 启动信号检测: {}", reason);

        Ok(LaunchSignal {
            m5_signal,
            m15_trend,
            h1_breakout,
            m1_deviation,
            m1_strong,
            all_confirmed,
            score,
            reason,
        })
    }

    // ==================== 私有检测方法 ====================

    /// 检测5m启动信号
    fn detect_5m_launch(&self, klines_5m: &[Kline]) -> Result<bool> {
        if klines_5m.len() < self.m5_consecutive_bullish + 5 {
            return Ok(false);
        }

        let recent: Vec<&Kline> = klines_5m
            .iter()
            .rev()
            .take(self.m5_consecutive_bullish)
            .collect();

        // 1. 检查连续阳线
        let all_bullish = recent.iter().all(|k| k.close > k.open);
        if !all_bullish {
            return Ok(false);
        }

        // 2. 检查实体大小
        let strong_body = recent.iter().all(|k| {
            let body_pct = ((k.close - k.open) / k.open) * 100.0;
            body_pct > self.m5_body_min_pct
        });
        if !strong_body {
            return Ok(false);
        }

        // 3. 检查成交量放大
        let volume_increased =
            self.check_volume_increase(klines_5m, self.m5_consecutive_bullish)?;

        Ok(all_bullish && strong_body && volume_increased)
    }

    /// 检查成交量放大
    fn check_volume_increase(&self, klines: &[Kline], recent_count: usize) -> Result<bool> {
        if klines.len() < recent_count + 5 {
            return Ok(true); // 数据不足,跳过检查
        }

        // 计算前5根的平均成交量
        let prev_5: Vec<&Kline> = klines.iter().rev().skip(recent_count).take(5).collect();
        let avg_volume_prev: f64 = prev_5.iter().map(|k| k.volume).sum::<f64>() / 5.0;

        // 计算最近N根的平均成交量
        let recent_n: Vec<&Kline> = klines.iter().rev().take(recent_count).collect();
        let avg_volume_recent: f64 =
            recent_n.iter().map(|k| k.volume).sum::<f64>() / recent_count as f64;

        // 计算增长率
        let volume_increase_pct = (avg_volume_recent - avg_volume_prev) / avg_volume_prev;

        Ok(volume_increase_pct > self.m5_volume_increase)
    }

    /// 确认15m趋势
    fn confirm_15m_trend(&self, klines_15m: &[Kline]) -> Result<bool> {
        if klines_15m.len() < self.m15_sma_period + 2 {
            return Ok(false);
        }

        // 1. 计算SMA20
        let sma_20 = self.calculate_sma(klines_15m, self.m15_sma_period);

        // 2. 检查最后一根K线在SMA20上方
        let last = klines_15m.last().unwrap();
        let above_sma = last.close > sma_20;

        // 3. 检查最近2根都是阳线
        let recent_2: Vec<&Kline> = klines_15m
            .iter()
            .rev()
            .take(self.m15_consecutive_bullish)
            .collect();
        let recent_2_bullish = recent_2.iter().all(|k| k.close > k.open);

        Ok(above_sma && recent_2_bullish)
    }

    /// 检查1h压力位突破
    fn check_1h_breakout(
        &self,
        klines_1h: &[Kline],
        entry_price: f64,
        current_price: f64,
    ) -> Result<bool> {
        if klines_1h.len() < self.h1_lookback {
            return Ok(false);
        }

        // 1. 找到最近5根1h的短期高点
        let recent_high: f64 = klines_1h
            .iter()
            .rev()
            .take(self.h1_lookback)
            .map(|k| k.high)
            .fold(f64::NEG_INFINITY, f64::max);

        // 2. 检查当前价格是否突破短期高点
        let breakout = current_price > recent_high;

        // 3. 检查涨幅是否>1.5%
        let gain_pct = ((current_price - entry_price) / entry_price) * 100.0;
        let sufficient_gain = gain_pct > self.h1_breakout_min_pct;

        Ok(breakout && sufficient_gain)
    }

    /// 计算1m实时偏离度
    fn calculate_1m_deviation(&self, klines_5m: &[Kline], current_price: f64) -> f64 {
        if klines_5m.is_empty() {
            return 0.0;
        }

        let last_5m_close = klines_5m.last().unwrap().close;
        ((current_price - last_5m_close) / last_5m_close) * 100.0
    }

    /// 计算简单移动平均
    fn calculate_sma(&self, klines: &[Kline], period: usize) -> f64 {
        if klines.len() < period {
            return klines.iter().map(|k| k.close).sum::<f64>() / klines.len() as f64;
        }

        let sum: f64 = klines.iter().rev().take(period).map(|k| k.close).sum();
        sum / period as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_kline(open: f64, close: f64, volume: f64) -> Kline {
        let high = open.max(close) * 1.01;
        let low = open.min(close) * 0.99;
        Kline {
            timestamp: 0,
            open,
            high,
            low,
            close,
            volume,
            ..Default::default()
        }
    }

    #[test]
    fn test_5m_launch_signal() {
        let detector = LaunchSignalDetector::default();

        // 构造5m K线: 前5根正常,后3根连续阳线+放量
        let mut klines = Vec::new();
        for _ in 0..5 {
            klines.push(create_test_kline(1.0, 1.01, 1000.0));
        }
        for i in 0..3 {
            klines.push(create_test_kline(
                1.0 + i as f64 * 0.01,
                1.01 + i as f64 * 0.01,
                1500.0, // 成交量放大50%
            ));
        }

        let result = detector.detect_5m_launch(&klines).unwrap();
        assert!(result);
    }

    #[test]
    fn test_15m_trend_confirmation() {
        let detector = LaunchSignalDetector::default();

        // 构造15m K线: 整体上升趋势
        let mut klines = Vec::new();
        for i in 0..25 {
            let open = 1.0 + i as f64 * 0.001;
            let close = open + 0.002;
            klines.push(create_test_kline(open, close, 1000.0));
        }

        let result = detector.confirm_15m_trend(&klines).unwrap();
        assert!(result);
    }

    #[test]
    fn test_1m_deviation_calculation() {
        let detector = LaunchSignalDetector::default();

        let klines = vec![create_test_kline(1.0, 1.005, 1000.0)];
        let current_price = 1.010; // 比5m收盘高0.5%

        let deviation = detector.calculate_1m_deviation(&klines, current_price);
        assert!((deviation - 0.497).abs() < 0.01); // 约0.5%
    }
}
