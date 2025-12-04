// 实时价格服务 - 使用 CoinGecko API
use anyhow::Result;
use log::{error, info};
use reqwest;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct CachedPrice {
    price: f64,
    timestamp: Instant,
}

pub struct PriceService {
    cache: Arc<RwLock<HashMap<String, CachedPrice>>>,
    cache_duration: Duration,
}

#[derive(Debug, Deserialize)]
struct CoinGeckoResponse {
    #[serde(flatten)]
    prices: HashMap<String, CoinGeckoPriceData>,
}

#[derive(Debug, Deserialize)]
struct CoinGeckoPriceData {
    usd: f64,
}

impl PriceService {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_duration: Duration::from_secs(60), // 缓存 60 秒
        }
    }

    /// 获取代币价格（带缓存）
    pub async fn get_price(&self, symbol: &str) -> Result<f64> {
        let symbol_lower = symbol.to_lowercase();

        // 检查缓存
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&symbol_lower) {
                if cached.timestamp.elapsed() < self.cache_duration {
                    return Ok(cached.price);
                }
            }
        }

        // 缓存过期或不存在，获取新价格
        let price = self.fetch_price(&symbol_lower).await?;

        // 更新缓存
        {
            let mut cache = self.cache.write().await;
            cache.insert(
                symbol_lower,
                CachedPrice {
                    price,
                    timestamp: Instant::now(),
                },
            );
        }

        Ok(price)
    }

    /// 批量获取多个代币价格
    pub async fn get_prices(&self, symbols: &[&str]) -> Result<HashMap<String, f64>> {
        let mut result = HashMap::new();

        // CoinGecko API 支持批量查询
        let coin_ids: Vec<String> = symbols
            .iter()
            .map(|s| self.symbol_to_coingecko_id(s))
            .collect();

        let ids_param = coin_ids.join(",");
        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd",
            ids_param
        );

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            error!("CoinGecko API 错误 {}: {}", status, body);
            return Err(anyhow::anyhow!("获取价格失败: {}", status));
        }

        let prices: CoinGeckoResponse = response.json().await?;

        // 映射回原始符号
        for (i, symbol) in symbols.iter().enumerate() {
            if let Some(price_data) = prices.prices.get(&coin_ids[i]) {
                result.insert(symbol.to_string(), price_data.usd);
                info!("✅ {} 价格: ${:.2}", symbol, price_data.usd);
            }
        }

        Ok(result)
    }

    /// 从缓存或 API 获取单个价格
    async fn fetch_price(&self, symbol: &str) -> Result<f64> {
        let coin_id = self.symbol_to_coingecko_id(symbol);
        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd",
            coin_id
        );

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            error!("CoinGecko API 错误: {}", response.status());
            return Err(anyhow::anyhow!("获取价格失败"));
        }

        let prices: CoinGeckoResponse = response.json().await?;

        prices
            .prices
            .get(&coin_id)
            .map(|p| {
                info!("✅ {} 实时价格: ${:.2}", symbol.to_uppercase(), p.usd);
                p.usd
            })
            .ok_or_else(|| anyhow::anyhow!("未找到 {} 的价格", symbol))
    }

    /// 将符号映射到 CoinGecko ID
    fn symbol_to_coingecko_id(&self, symbol: &str) -> String {
        match symbol.to_lowercase().as_str() {
            "btc" | "bitcoin" => "bitcoin",
            "eth" | "ethereum" => "ethereum",
            "bnb" => "binancecoin",
            "sol" | "solana" => "solana",
            "usdt" | "tether" => "tether",
            "usdc" => "usd-coin",
            "busd" => "binance-usd",
            "ada" | "cardano" => "cardano",
            "xrp" | "ripple" => "ripple",
            "dot" | "polkadot" => "polkadot",
            "doge" | "dogecoin" => "dogecoin",
            "matic" | "polygon" => "matic-network",
            "avax" | "avalanche" => "avalanche-2",
            "link" | "chainlink" => "chainlink",
            "atom" | "cosmos" => "cosmos",
            _ => symbol,
        }
        .to_string()
    }

    /// 清空缓存
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        info!("价格缓存已清空");
    }
}

impl Default for PriceService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_price() {
        let service = PriceService::new();
        let price = service.get_price("sol").await.unwrap();
        assert!(price > 0.0);
        println!("SOL 价格: ${}", price);
    }

    #[tokio::test]
    async fn test_get_prices() {
        let service = PriceService::new();
        let prices = service.get_prices(&["btc", "eth", "sol"]).await.unwrap();
        assert_eq!(prices.len(), 3);
        println!("价格: {:?}", prices);
    }
}
