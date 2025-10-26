use anyhow::Result;
use dotenv::dotenv;
use rust_trading_bot::binance_client::BinanceClient;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    println!("🧪 手动测试开空单\n");

    let api_key = env::var("BINANCE_API_KEY")?;
    let secret = env::var("BINANCE_SECRET_KEY")?;
    let testnet = env::var("BINANCE_TESTNET")?.parse::<bool>()?;

    let symbol = "SUPERUSDT";
    let leverage = env::var("LEVERAGE")
        .unwrap_or_else(|_| "15".to_string())
        .parse::<u32>()
        .unwrap_or(15);
    let margin = env::var("COPY_MARGIN_USDT")
        .unwrap_or_else(|_| "2".to_string())
        .parse::<f64>()
        .unwrap_or(2.0);
    let margin_type = env::var("TRADING_MARGIN_TYPE").unwrap_or_else(|_| "ISOLATED".to_string());
    let dual_side_position = matches!(
        env::var("POSITION_MODE")
            .unwrap_or_else(|_| "SINGLE".to_string())
            .to_uppercase()
            .as_str(),
        "DUAL"
    );

    println!("📊 交易对: {}", symbol);
    println!("⚡ 杠杆: {}x", leverage);
    println!("💰 保证金: {} USDT", margin);
    println!("🌐 环境: {}\n", if testnet { "测试网" } else { "主网" });

    let client = BinanceClient::new(api_key, secret, testnet);

    // 查询当前价格
    println!("📡 查询当前价格...");
    let price = client.get_current_price(symbol).await?;
    println!("   当前价格: {} USDT\n", price);

    // 查询交易规则并计算数量
    println!("📐 计算数量(依据交易规则)...");
    let rules = client.get_symbol_trading_rules(symbol).await?;
    let quantity = client.calculate_quantity_with_margin(price, margin, leverage, &rules)?;
    println!("   保证金: {} USDT", margin);
    println!("   杠杆: {}x", leverage);
    println!("   数量 (stepSize= {}): {:.6}\n", rules.step_size, quantity);

    // 开空单
    println!("📉 执行开空单 (逐仓: {}, 持仓模式: {})...", margin_type, if dual_side_position { "双向" } else { "单向" });
    client
        .open_short(symbol, quantity, leverage, &margin_type, dual_side_position)
        .await?;
    println!("✅ 开空成功!\n");

    // 查询持仓
    println!("📦 查询持仓...");
    let positions = client.get_positions().await?;

    if let Some(pos) = positions.iter().find(|p| p.symbol == symbol) {
        println!("✅ 找到持仓:");
        println!("   交易对: {}", pos.symbol);
        println!("   方向: {}", pos.side);
        println!("   数量: {:.4}", pos.size);
        println!("   入场价: {}", pos.entry_price);
        println!("   杠杆: {}x", pos.leverage);
        println!("   未实现盈亏: {} USDT", pos.pnl);
    } else {
        println!("❌ 未找到持仓");
    }

    Ok(())
}
