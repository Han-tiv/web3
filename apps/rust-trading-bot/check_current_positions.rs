use rust_trading_bot::binance_client::BinanceClient;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    
    let api_key = env::var("BINANCE_API_KEY")?;
    let secret_key = env::var("BINANCE_SECRET_KEY")?;
    let testnet = env::var("BINANCE_TESTNET")?.parse::<bool>()?;
    
    let client = BinanceClient::new(api_key, secret_key, testnet);
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 当前持仓查询");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    match client.get_positions().await {
        Ok(positions) => {
            if positions.is_empty() {
                println!("✅ 当前没有持仓\n");
            } else {
                println!("✅ 找到 {} 个持仓:\n", positions.len());
                for (i, pos) in positions.iter().enumerate() {
                    println!("持仓 {}:", i + 1);
                    println!("  币种: {}", pos.symbol);
                    println!("  方向: {}", pos.side);
                    println!("  数量: {}", pos.size);
                    println!("  入场价: ${:.4}", pos.entry_price);
                    println!("  标记价: ${:.4}", pos.mark_price);
                    println!("  盈亏: ${:.4}", pos.pnl);
                    println!("  杠杆: {}x", pos.leverage);
                    
                    let pnl_pct = (pos.pnl / (pos.entry_price * pos.size / pos.leverage as f64)) * 100.0;
                    println!("  盈亏%: {:.2}%\n", pnl_pct);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ 查询持仓失败: {}", e);
        }
    }
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📋 当前挂单查询");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // 查询所有挂单
    println!("⚠️  挂单查询功能需要额外实现\n");
    
    Ok(())
}
