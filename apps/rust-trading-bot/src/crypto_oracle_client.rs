// ===================================
// 此模块已废弃 - 不再使用 CryptoOracle 情绪数据
// 改用纯技术指标版本（指标Plus版）
// ===================================

use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use log::{info, warn};
use chrono::{DateTime, Utc, Duration};

/// CryptoOracle API 响应结构
#[derive(Debug, Deserialize)]
struct CryptoOracleResponse {
    code: i32,
    message: String,
    data: Option<Vec<DataItem>>,
}

#[derive(Debug, Deserialize)]
struct DataItem {
    #[serde(rename = "timePeriods")]
    time_periods: Vec<TimePeriod>,
}

#[derive(Debug, Deserialize)]
struct TimePeriod {
    #[serde(rename = "startTime")]
    start_time: String,
    #[serde(rename = "endTime")]
    end_time: String,
    data: Vec<EndpointData>,
}

#[derive(Debug, Deserialize)]
struct EndpointData {
    endpoint: String,
    value: String,
}

/// CryptoOracle API 请求体
#[derive(Debug, Serialize)]
struct CryptoOracleRequest {
    #[serde(rename = "apiKey")]
    api_key: String,
    endpoints: Vec<String>,
    #[serde(rename = "startTime")]
    start_time: String,
    #[serde(rename = "endTime")]
    end_time: String,
    #[serde(rename = "timeType")]
    time_type: String,
    token: Vec<String>,
}

/// 市场情绪数据
#[derive(Debug, Clone)]
pub struct CryptoOracleSentiment {
    pub positive_ratio: f64,
    pub negative_ratio: f64,
    pub net_sentiment: f64,
    pub data_time: String,
    pub data_delay_minutes: i64,
}

pub struct CryptoOracleClient {
    client: Client,
    api_key: String,
    api_url: String,
}

impl CryptoOracleClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            api_url: "https://service.cryptoracle.network/openapi/v2/endpoint".to_string(),
        }
    }

    /// 获取市场情绪数据
    pub async fn get_sentiment(&self, symbol: &str) -> Result<CryptoOracleSentiment> {
        match self.fetch_sentiment(symbol).await {
            Ok(sentiment) => Ok(sentiment),
            Err(e) => {
                warn!("⚠️  获取 CryptoOracle 情绪数据失败: {}, 使用默认值", e);
                // 返回中性情绪数据
                Ok(CryptoOracleSentiment {
                    positive_ratio: 0.5,
                    negative_ratio: 0.5,
                    net_sentiment: 0.0,
                    data_time: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    data_delay_minutes: 0,
                })
            }
        }
    }

    async fn fetch_sentiment(&self, symbol: &str) -> Result<CryptoOracleSentiment> {
        let end_time = Utc::now();
        let start_time = end_time - Duration::hours(4);

        let request_body = CryptoOracleRequest {
            api_key: self.api_key.clone(),
            endpoints: vec![
                "CO-A-02-01".to_string(),  // 乐观比例
                "CO-A-02-02".to_string(),  // 悲观比例
            ],
            start_time: start_time.format("%Y-%m-%d %H:%M:%S").to_string(),
            end_time: end_time.format("%Y-%m-%d %H:%M:%S").to_string(),
            time_type: "15m".to_string(),
            token: vec![symbol.to_string()],
        };

        let response = self
            .client
            .post(&self.api_url)
            .header("Content-Type", "application/json")
            .header("X-API-KEY", &self.api_key)
            .json(&request_body)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("CryptoOracle API 返回状态: {}", response.status());
        }

        let data: CryptoOracleResponse = response.json().await?;

        if data.code != 200 {
            anyhow::bail!("CryptoOracle API 返回错误: {}", data.message);
        }

        let data_items = data.data.ok_or_else(|| anyhow::anyhow!("没有数据"))?;
        
        if data_items.is_empty() {
            anyhow::bail!("数据为空");
        }

        // 查找第一个有有效数据的时间段
        for time_period in &data_items[0].time_periods {
            let mut positive_ratio = None;
            let mut negative_ratio = None;

            for endpoint_data in &time_period.data {
                let value = endpoint_data.value.trim();
                if value.is_empty() {
                    continue;
                }

                match endpoint_data.endpoint.as_str() {
                    "CO-A-02-01" => {
                        if let Ok(val) = value.parse::<f64>() {
                            positive_ratio = Some(val);
                        }
                    }
                    "CO-A-02-02" => {
                        if let Ok(val) = value.parse::<f64>() {
                            negative_ratio = Some(val);
                        }
                    }
                    _ => {}
                }
            }

            // 如果找到完整数据
            if let (Some(positive), Some(negative)) = (positive_ratio, negative_ratio) {
                let net_sentiment = positive - negative;
                
                // 计算数据延迟
                let data_time = DateTime::parse_from_str(
                    &format!("{} +00:00", time_period.start_time),
                    "%Y-%m-%d %H:%M:%S %z"
                ).map(|dt| dt.with_timezone(&Utc))?;
                
                let data_delay_minutes = (Utc::now() - data_time).num_minutes();

                info!("✅ 使用情绪数据时间: {} (延迟: {}分钟)", time_period.start_time, data_delay_minutes);

                return Ok(CryptoOracleSentiment {
                    positive_ratio: positive,
                    negative_ratio: negative,
                    net_sentiment,
                    data_time: time_period.start_time.clone(),
                    data_delay_minutes,
                });
            }
        }

        anyhow::bail!("所有时间段数据都为空")
    }

    /// 格式化情绪数据为可读文本
    pub fn format_sentiment(&self, sentiment: &CryptoOracleSentiment) -> String {
        let sign = if sentiment.net_sentiment >= 0.0 { "+" } else { "" };
        format!(
            "【市场情绪】乐观{:.1}% 悲观{:.1}% 净值{}{}",
            sentiment.positive_ratio * 100.0,
            sentiment.negative_ratio * 100.0,
            sign,
            sentiment.net_sentiment
        )
    }

    /// 解读情绪指标
    pub fn interpret_sentiment(&self, sentiment: &CryptoOracleSentiment) -> String {
        if sentiment.net_sentiment > 0.1 {
            "市场乐观情绪较强".to_string()
        } else if sentiment.net_sentiment < -0.1 {
            "市场悲观情绪较强".to_string()
        } else {
            "市场情绪中性".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_crypto_oracle_client() {
        let api_key = std::env::var("CRYPTO_ORACLE_API_KEY")
            .unwrap_or_else(|_| "test_key".to_string());
        
        let client = CryptoOracleClient::new(api_key);
        
        // 测试获取情绪数据（即使失败也会返回默认值）
        let result = client.get_sentiment("BTC").await;
        assert!(result.is_ok());
    }
}
