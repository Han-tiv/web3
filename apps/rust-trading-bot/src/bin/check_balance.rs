use anyhow::Result;
use dotenv::dotenv;
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use std::env;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct AccountInfo {
    totalWalletBalance: String,
    availableBalance: String,
    totalUnrealizedProfit: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct Position {
    symbol: String,
    positionAmt: String,
    entryPrice: String,
    markPrice: String,
    unRealizedProfit: String,
    leverage: String,
}

fn sign_request(query: &str, secret: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(query.as_bytes());
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

async fn get_account_info(api_key: &str, secret_key: &str, testnet: bool) -> Result<AccountInfo> {
    let base_url = if testnet {
        "https://testnet.binancefuture.com"
    } else {
        "https://fapi.binance.com"
    };

    let timestamp = chrono::Utc::now().timestamp_millis();
    let query = format!("timestamp={}", timestamp);
    let signature = sign_request(&query, secret_key);
    let url = format!(
        "{}/fapi/v2/account?{}&signature={}",
        base_url, query, signature
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("X-MBX-APIKEY", api_key)
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await?;

    if !status.is_success() {
        return Err(anyhow::anyhow!("API错误 ({}): {}", status, body));
    }

    let resp: AccountInfo = serde_json::from_str(&body)?;
    Ok(resp)
}

async fn get_positions(api_key: &str, secret_key: &str, testnet: bool) -> Result<Vec<Position>> {
    let base_url = if testnet {
        "https://testnet.binancefuture.com"
    } else {
        "https://fapi.binance.com"
    };

    let timestamp = chrono::Utc::now().timestamp_millis();
    let query = format!("timestamp={}", timestamp);
    let signature = sign_request(&query, secret_key);
    let url = format!(
        "{}/fapi/v2/positionRisk?{}&signature={}",
        base_url, query, signature
    );

    let client = reqwest::Client::new();
    let positions: Vec<Position> = client
        .get(&url)
        .header("X-MBX-APIKEY", api_key)
        .send()
        .await?
        .json()
        .await?;

    Ok(positions
        .into_iter()
        .filter(|p| p.positionAmt.parse::<f64>().unwrap_or(0.0).abs() > 0.0)
        .collect())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    println!("🚀 Binance账户余额查询工具\n");

    let api_key = env::var("BINANCE_API_KEY").unwrap_or_else(|_| {
        println!("⚠️  未设置 BINANCE_API_KEY");
        std::process::exit(1);
    });

    let secret_key = env::var("BINANCE_SECRET").unwrap_or_else(|_| {
        println!("⚠️  未设置 BINANCE_SECRET");
        std::process::exit(1);
    });

    let testnet = env::var("BINANCE_TESTNET")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);

    println!(
        "📡 连接到 Binance {}",
        if testnet { "测试网" } else { "主网" }
    );
    println!("════════════════════════════════════════\n");

    match get_account_info(&api_key, &secret_key, testnet).await {
        Ok(account) => {
            println!("✅ 账户信息获取成功!\n");
            println!("💰 账户余额信息:");
            println!("   总余额: {} USDT", account.totalWalletBalance);
            println!("   可用余额: {} USDT", account.availableBalance);
            println!("   未实现盈亏: {} USDT", account.totalUnrealizedProfit);
            println!("\n════════════════════════════════════════\n");

            match get_positions(&api_key, &secret_key, testnet).await {
                Ok(positions) => {
                    if positions.is_empty() {
                        println!("📦 当前持仓: 无");
                    } else {
                        println!("📦 当前持仓 ({} 个):\n", positions.len());
                        for (i, pos) in positions.iter().enumerate() {
                            let amt = pos.positionAmt.parse::<f64>().unwrap_or(0.0);
                            let pnl = pos.unRealizedProfit.parse::<f64>().unwrap_or(0.0);
                            let pnl_emoji = if pnl > 0.0 { "🟢" } else { "🔴" };
                            let side_emoji = if amt > 0.0 { "📈" } else { "📉" };
                            let side = if amt > 0.0 { "LONG" } else { "SHORT" };

                            println!("   {}. {} {}", i + 1, side_emoji, pos.symbol);
                            println!("      方向: {}", side);
                            println!("      数量: {}", amt.abs());
                            println!("      入场价: ${}", pos.entryPrice);
                            println!("      标记价: ${}", pos.markPrice);
                            println!("      未实现盈亏: ${:.2} {}", pnl, pnl_emoji);
                            println!("      杠杆: {}x", pos.leverage);
                            println!();
                        }

                        let total_pnl: f64 = positions
                            .iter()
                            .map(|p| p.unRealizedProfit.parse::<f64>().unwrap_or(0.0))
                            .sum();
                        let total_pnl_emoji = if total_pnl > 0.0 { "🟢" } else { "🔴" };
                        println!("   📊 总盈亏: ${:.2} {}", total_pnl, total_pnl_emoji);
                    }
                }
                Err(e) => {
                    println!("⚠️  获取持仓失败: {}", e);
                }
            }

            println!("\n════════════════════════════════════════");
            println!("✅ 查询完成");
        }
        Err(e) => {
            println!("❌ 账户信息获取失败: {}", e);
            println!("\n💡 可能的原因:");
            println!("   1. API Key 或 Secret Key 错误");
            println!("   2. API权限不足（需要期货交易权限）");
            println!("   3. IP白名单限制");
            println!("   4. 网络连接问题");
            std::process::exit(1);
        }
    }

    Ok(())
}
