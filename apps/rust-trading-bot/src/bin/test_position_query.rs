use std::{env, time::Duration};

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use hmac::{Hmac, Mac};
use log::{info, warn};
use reqwest::{Client, StatusCode};
use rust_trading_bot::{
    binance_client::BinanceClient,
    exchange_trait::{ExchangeClient, Position},
};
use serde_json::Value;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

struct RawPositionResponse {
    status: StatusCode,
    body: String,
    json: Option<Value>,
}

/// 构建Base URL，允许通过环境变量覆盖。
fn resolve_papi_base_url(testnet: bool) -> String {
    if let Ok(url) = env::var("BINANCE_PAPI_BASE_URL") {
        return url;
    }
    if testnet {
        "https://testnet.binancefuture.com".to_string()
    } else {
        "https://papi.binance.com".to_string()
    }
}

fn sign_request(secret_key: &str, payload: &str) -> Result<String> {
    let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes()).context("初始化HMAC失败")?;
    mac.update(payload.as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

async fn fetch_raw_positions(
    client: &Client,
    api_key: &str,
    secret_key: &str,
    papi_base_url: &str,
    symbol: Option<&str>,
) -> Result<RawPositionResponse> {
    let timestamp = Utc::now().timestamp_millis();
    let mut query_parts = vec![format!("timestamp={timestamp}")];
    if let Some(sym) = symbol {
        query_parts.insert(0, format!("symbol={}", sym.to_uppercase()));
    }
    let query = query_parts.join("&");
    let signature = sign_request(secret_key, &query)?;
    let endpoint = format!(
        "{}/papi/v1/um/positionRisk?{}&signature={}",
        papi_base_url.trim_end_matches('/'),
        query,
        signature
    );

    info!("📡 请求URL: {}", endpoint);

    let response = client
        .get(&endpoint)
        .header("X-MBX-APIKEY", api_key)
        .send()
        .await
        .context("请求PAPI持仓接口失败")?;
    let status = response.status();
    let body = response.text().await.context("读取PAPI持仓响应失败")?;
    let json = serde_json::from_str::<Value>(&body).ok();

    Ok(RawPositionResponse { status, body, json })
}

fn describe_response_format(json: &Value) {
    match json {
        Value::Array(arr) => {
            println!("🧠 响应格式: 数组 (记录数: {})", arr.len());
        }
        Value::Object(map) => {
            if let Some(data) = map.get("data") {
                match data {
                    Value::Array(arr) => {
                        println!("🧠 响应格式: 包裹(data数组)，记录数: {}", arr.len());
                    }
                    Value::Object(obj) => {
                        println!("🧠 响应格式: 包裹(data对象)，键数量: {}", obj.len());
                    }
                    other => {
                        println!("🧠 响应格式: 包裹(data类型: {})", value_type_name(other));
                    }
                }
            } else {
                println!("🧠 响应格式: 映射 (交易对数量: {})", map.len());
            }
        }
        other => {
            println!("🧠 响应格式: 未知 ({})", value_type_name(other));
        }
    }
}

fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn print_positions(positions: &[Position]) {
    if positions.is_empty() {
        println!("ℹ️ 当前无任何持仓。");
        return;
    }

    println!("✅ 成功获取持仓数量: {}", positions.len());
    for pos in positions {
        println!(
            "  - {}: {:.6} {} (入场价: {:.4}, 标记价: {:.4}, 未实现盈亏: {:.4}, 杠杆: {}, 保证金: {:.4})",
            pos.symbol,
            pos.size,
            pos.side,
            pos.entry_price,
            pos.mark_price,
            pos.pnl,
            pos.leverage,
            pos.margin
        );
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 允许重复初始化日志器，避免与其他二进制冲突。
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .try_init();

    let api_key = env::var("BINANCE_API_KEY").context("缺少环境变量 BINANCE_API_KEY")?;
    let secret_key = env::var("BINANCE_API_SECRET").context("缺少环境变量 BINANCE_API_SECRET")?;
    let testnet = env::var("BINANCE_TESTNET")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .map_err(|e| anyhow!("解析 BINANCE_TESTNET 失败: {e}"))?;
    let papi_base_url = resolve_papi_base_url(testnet);

    println!("🔧 使用PAPI Base URL: {}", papi_base_url);
    println!("🔍 查询所有持仓...\n");

    let http_client = Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .context("构建HTTP客户端失败")?;

    let client = BinanceClient::new(api_key.clone(), secret_key.clone(), testnet);

    let raw_all =
        fetch_raw_positions(&http_client, &api_key, &secret_key, &papi_base_url, None).await?;
    println!("HTTP状态: {}", raw_all.status);
    println!("📦 原始响应文本:\n{}\n", raw_all.body);
    if let Some(json) = &raw_all.json {
        match serde_json::to_string_pretty(json) {
            Ok(pretty) => {
                println!("🧾 原始JSON结构:\n{}\n", pretty);
            }
            Err(err) => {
                println!("⚠️ 无法格式化JSON: {err}");
            }
        }
        describe_response_format(json);
    } else {
        println!("⚠️ 原始响应不是有效JSON，跳过格式解析");
    }

    match client.get_positions().await {
        Ok(positions) => print_positions(&positions),
        Err(err) => println!("❌ 获取解析后的持仓失败: {err:?}"),
    }

    println!("\n🔍 测试查询XRPUSDT持仓...");

    let raw_symbol = fetch_raw_positions(
        &http_client,
        &api_key,
        &secret_key,
        &papi_base_url,
        Some("XRPUSDT"),
    )
    .await?;
    println!("HTTP状态: {}", raw_symbol.status);
    println!("📦 XRPUSDT原始响应:\n{}\n", raw_symbol.body);
    if let Some(json) = &raw_symbol.json {
        match serde_json::to_string_pretty(json) {
            Ok(pretty) => {
                println!("🧾 XRPUSDT JSON结构:\n{}\n", pretty);
            }
            Err(err) => {
                println!("⚠️ 无法格式化XRPUSDT JSON: {err}");
            }
        }
        describe_response_format(json);
    } else {
        println!("⚠️ XRPUSDT响应不是有效JSON，跳过格式解析");
    }

    match client.get_position("XRPUSDT").await {
        Ok(Some(position)) => {
            println!("✅ XRPUSDT持仓: {:?}", position);
        }
        Ok(None) => {
            println!("ℹ️ 当前无XRPUSDT持仓。");
        }
        Err(err) => {
            println!("⚠️ 查询XRPUSDT失败: {err:?}");
        }
    }

    Ok(())
}
