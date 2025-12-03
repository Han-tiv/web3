use anyhow::Result;
use dotenv::dotenv;
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use std::env;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct UnifiedBalance {
    asset: String,
    totalWalletBalance: String,
    umWalletBalance: String,
    umUnrealizedPNL: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct UnifiedPosition {
    symbol: String,
    positionAmt: String,
    entryPrice: String,
    markPrice: String,
    unRealizedProfit: String,
    leverage: String,
    positionSide: String,
}

fn sign_request(query: &str, secret: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(query.as_bytes());
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

async fn get_unified_balance(api_key: &str, secret_key: &str) -> Result<Vec<UnifiedBalance>> {
    let base_url = "https://papi.binance.com";

    let timestamp = chrono::Utc::now().timestamp_millis();
    let query = format!("timestamp={}", timestamp);
    let signature = sign_request(&query, secret_key);
    let url = format!(
        "{}/papi/v1/balance?{}&signature={}",
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

    let balances: Vec<UnifiedBalance> = serde_json::from_str(&body)?;
    Ok(balances)
}

async fn get_unified_positions(api_key: &str, secret_key: &str) -> Result<Vec<UnifiedPosition>> {
    let base_url = "https://papi.binance.com";

    let timestamp = chrono::Utc::now().timestamp_millis();
    let query = format!("timestamp={}", timestamp);
    let signature = sign_request(&query, secret_key);
    let url = format!(
        "{}/papi/v1/um/positionRisk?{}&signature={}",
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

    let positions: Vec<UnifiedPosition> = serde_json::from_str(&body)?;

    Ok(positions
        .into_iter()
        .filter(|p| p.positionAmt.parse::<f64>().unwrap_or(0.0).abs() > 0.0)
        .collect())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    println!("🚀 Binance 统一账户余额查询工具\n");

    let api_key = env::var("BINANCE_API_KEY").unwrap_or_else(|_| {
        println!("⚠️  未设置 BINANCE_API_KEY");
        std::process::exit(1);
    });

    let secret_key = env::var("BINANCE_SECRET").unwrap_or_else(|_| {
        println!("⚠️  未设置 BINANCE_SECRET");
        std::process::exit(1);
    });

    println!("📡 连接到 Binance 统一账户 (Portfolio Margin)");
    println!("════════════════════════════════════════\n");

    match get_unified_balance(&api_key, &secret_key).await {
        Ok(balances) => {
            println!("✅ 账户余额获取成功!\n");
            println!("💰 账户余额信息:");

            let mut total_balance = 0.0;
            let mut total_available = 0.0;
            let mut total_unpnl = 0.0;

            for balance in &balances {
                let wallet = balance.totalWalletBalance.parse::<f64>().unwrap_or(0.0);
                let um_wallet = balance.umWalletBalance.parse::<f64>().unwrap_or(0.0);
                let unpnl = balance.umUnrealizedPNL.parse::<f64>().unwrap_or(0.0);

                if wallet > 0.01 || um_wallet > 0.01 || unpnl.abs() > 0.01 {
                    println!("\n   币种: {}", balance.asset);
                    println!("   总余额: {}", balance.totalWalletBalance);
                    println!("   U本位合约余额: {}", balance.umWalletBalance);
                    println!("   未实现盈亏: {}", balance.umUnrealizedPNL);

                    // 如果是 USDT，累加到总计
                    if balance.asset == "USDT" {
                        total_balance = wallet;
                        total_available = um_wallet;
                        total_unpnl = unpnl;
                    }
                }
            }

            println!("\n════════════════════════════════════════");
            println!("\n📊 USDT 汇总:");
            println!("   总钱包余额: {:.2} USDT", total_balance);
            println!("   U本位合约余额: {:.2} USDT", total_available);
            let unpnl_emoji = if total_unpnl > 0.0 {
                "🟢"
            } else if total_unpnl < 0.0 {
                "🔴"
            } else {
                "⚪"
            };
            println!("   未实现盈亏: {:.2} USDT {}", total_unpnl, unpnl_emoji);

            println!("\n════════════════════════════════════════\n");

            match get_unified_positions(&api_key, &secret_key).await {
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

                            println!(
                                "   {}. {} {} ({})",
                                i + 1,
                                side_emoji,
                                pos.symbol,
                                pos.positionSide
                            );
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
            println!("❌ 账户余额获取失败: {}", e);
            println!("\n💡 可能的原因:");
            println!("   1. API Key 或 Secret 错误");
            println!("   2. API权限不足（需要统一账户权限）");
            println!("   3. IP白名单限制");
            println!("   4. 网络连接问题");
            println!("   5. 不是统一账户（Portfolio Margin Account）");
            std::process::exit(1);
        }
    }

    Ok(())
}
