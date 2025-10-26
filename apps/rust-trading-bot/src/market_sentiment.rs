use reqwest::Client;
use serde::Deserialize;
use anyhow::Result;
use log::{info, warn};
use crate::deepseek_client::MarketSentiment;

#[derive(Debug, Deserialize)]
struct FearGreedResponse {
    data: Vec<FearGreedData>,
}

#[derive(Debug, Deserialize)]
struct FearGreedData {
    value: String,
    value_classification: String,
    timestamp: String,
}

pub struct SentimentAnalyzer {
    client: Client,
}

impl SentimentAnalyzer {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// 获取市场情绪数据
    pub async fn get_market_sentiment(
        &self,
        current_price: f64,
        price_24h_ago: f64,
    ) -> Result<MarketSentiment> {
        let fear_greed = self.get_fear_greed_index().await?;
        let price_change = ((current_price - price_24h_ago) / price_24h_ago) * 100.0;

        info!("😊 市场情绪: {} ({}) | 24h变化: {:+.2}%",
            fear_greed.0, fear_greed.1, price_change
        );

        Ok(MarketSentiment {
            fear_greed_value: fear_greed.0,
            fear_greed_label: fear_greed.1,
            price_change_24h: price_change,
            long_short_ratio: 1.0, // 需要从交易所 API 获取
        })
    }

    /// 获取恐慌贪婪指数
    async fn get_fear_greed_index(&self) -> Result<(i32, String)> {
        match self.fetch_fear_greed().await {
            Ok(data) => Ok(data),
            Err(e) => {
                warn!("⚠️  获取恐慌贪婪指数失败: {}, 使用默认值", e);
                Ok((50, "Neutral".to_string()))
            }
        }
    }

    async fn fetch_fear_greed(&self) -> Result<(i32, String)> {
        let response = self
            .client
            .get("https://api.alternative.me/fng/?limit=1")
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Fear & Greed API returned status: {}", response.status());
        }

        let data: FearGreedResponse = response.json().await?;

        if let Some(latest) = data.data.first() {
            let value = latest.value.parse::<i32>().unwrap_or(50);
            let label = latest.value_classification.clone();
            Ok((value, label))
        } else {
            anyhow::bail!("No data in Fear & Greed response");
        }
    }

    /// 解释恐慌贪婪指数
    pub fn interpret_fear_greed(&self, value: i32) -> String {
        match value {
            0..=24 => "极度恐慌 - 可能是买入机会".to_string(),
            25..=44 => "恐慌 - 市场悲观".to_string(),
            45..=55 => "中性 - 市场平衡".to_string(),
            56..=75 => "贪婪 - 市场乐观".to_string(),
            76..=100 => "极度贪婪 - 可能是卖出机会".to_string(),
            _ => "未知".to_string(),
        }
    }

    /// 根据价格变化判断市场动能
    pub fn analyze_momentum(&self, price_change_24h: f64) -> String {
        if price_change_24h > 5.0 {
            "强劲上涨动能".to_string()
        } else if price_change_24h > 2.0 {
            "温和上涨".to_string()
        } else if price_change_24h > -2.0 {
            "横盘整理".to_string()
        } else if price_change_24h > -5.0 {
            "温和下跌".to_string()
        } else {
            "强劲下跌动能".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpret_fear_greed() {
        let analyzer = SentimentAnalyzer::new();
        
        assert_eq!(analyzer.interpret_fear_greed(10), "极度恐慌 - 可能是买入机会");
        assert_eq!(analyzer.interpret_fear_greed(35), "恐慌 - 市场悲观");
        assert_eq!(analyzer.interpret_fear_greed(50), "中性 - 市场平衡");
        assert_eq!(analyzer.interpret_fear_greed(65), "贪婪 - 市场乐观");
        assert_eq!(analyzer.interpret_fear_greed(85), "极度贪婪 - 可能是卖出机会");
    }

    #[test]
    fn test_analyze_momentum() {
        let analyzer = SentimentAnalyzer::new();
        
        assert_eq!(analyzer.analyze_momentum(6.0), "强劲上涨动能");
        assert_eq!(analyzer.analyze_momentum(3.0), "温和上涨");
        assert_eq!(analyzer.analyze_momentum(0.0), "横盘整理");
        assert_eq!(analyzer.analyze_momentum(-3.0), "温和下跌");
        assert_eq!(analyzer.analyze_momentum(-6.0), "强劲下跌动能");
    }
}
