use anyhow::Result;
use dotenv::dotenv;
use hmac::{Hmac, Mac};
use reqwest;
use sha2::Sha256;
use std::env;

type HmacSha256 = Hmac<Sha256>;

fn sign_request(query: &str, secret: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(query.as_bytes());
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    println!("🔍 Binance 统一账户 API 调试工具\n");

    let api_key = env::var("BINANCE_API_KEY").unwrap_or_else(|_| {
        println!("⚠️  未设置 BINANCE_API_KEY");
        std::process::exit(1);
    });

    let secret_key = env::var("BINANCE_SECRET").unwrap_or_else(|_| {
        println!("⚠️  未设置 BINANCE_SECRET");
        std::process::exit(1);
    });

    println!("测试 1: /papi/v1/balance");
    println!("════════════════════════════════════════\n");

    let base_url = "https://papi.binance.com";
    let timestamp = chrono::Utc::now().timestamp_millis();
    let query = format!("timestamp={}", timestamp);
    let signature = sign_request(&query, &secret_key);
    let url = format!(
        "{}/papi/v1/balance?{}&signature={}",
        base_url, query, signature
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("X-MBX-APIKEY", &api_key)
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await?;

    println!("状态码: {}", status);
    println!("响应内容:\n{}\n", body);

    println!("════════════════════════════════════════\n");
    println!("测试 2: /papi/v1/account");
    println!("════════════════════════════════════════\n");

    let timestamp = chrono::Utc::now().timestamp_millis();
    let query = format!("timestamp={}", timestamp);
    let signature = sign_request(&query, &secret_key);
    let url = format!(
        "{}/papi/v1/account?{}&signature={}",
        base_url, query, signature
    );

    let response = client
        .get(&url)
        .header("X-MBX-APIKEY", &api_key)
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await?;

    println!("状态码: {}", status);
    println!("响应内容:\n{}\n", body);

    println!("════════════════════════════════════════\n");
    println!("测试 3: /papi/v1/um/positionRisk");
    println!("════════════════════════════════════════\n");

    let timestamp = chrono::Utc::now().timestamp_millis();
    let query = format!("timestamp={}", timestamp);
    let signature = sign_request(&query, &secret_key);
    let url = format!(
        "{}/papi/v1/um/positionRisk?{}&signature={}",
        base_url, query, signature
    );

    let response = client
        .get(&url)
        .header("X-MBX-APIKEY", &api_key)
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await?;

    println!("状态码: {}", status);
    println!("响应内容:\n{}\n", body);

    Ok(())
}
