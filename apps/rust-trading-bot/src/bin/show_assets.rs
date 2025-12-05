// 资产展示工具 - 显示所有交易所的余额、持仓和收益
use anyhow::Result;
use dotenv::dotenv;
use std::env;
use std::sync::Arc;

// 已删除的交易所客户端已注释
// use rust_trading_bot::bitget_client::BitgetClient;
// use rust_trading_bot::bsc_wallet::BscWallet;
// use rust_trading_bot::bybit_client::BybitClient;
use rust_trading_bot::exchange_trait::ExchangeClient;
// use rust_trading_bot::gate_client::GateClient;
use rust_trading_bot::hyperliquid_client::HyperliquidClient;
// use rust_trading_bot::okx_client::OkxClient;
// use rust_trading_bot::solana_wallet::SolanaWallet;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║          💎 多交易所资产监控面板 💎                          ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    // 创建所有交易所客户端
    let mut exchanges: Vec<Arc<dyn ExchangeClient>> = Vec::new();

    // Binance
    if let (Ok(key), Ok(secret)) = (env::var("BINANCE_API_KEY"), env::var("BINANCE_SECRET")) {
        let testnet = env::var("BINANCE_TESTNET")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);
        let client = rust_trading_bot::binance_client::BinanceClient::new(key, secret, testnet);
        exchanges.push(Arc::new(client));
    }

    /* OKX已被删除
    // OKX
    if let (Ok(key), Ok(secret), Ok(passphrase)) = (
        env::var("OKX_API_KEY"),
        env::var("OKX_SECRET"),
        env::var("OKX_PASSPHRASE"),
    ) {
        let testnet = env::var("OKX_TESTNET")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);
        let client = OkxClient::new(key, secret, passphrase, testnet);
        exchanges.push(Arc::new(client));
    }
    */

    /* Bitget已被删除
    // Bitget
    if let (Ok(key), Ok(secret), Ok(passphrase)) = (
        env::var("BITGET_API_KEY"),
        env::var("BITGET_SECRET"),
        env::var("BITGET_PASSPHRASE"),
    ) {
        let testnet = env::var("BITGET_TESTNET")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);
        let client = BitgetClient::new(key, secret, passphrase, testnet);
        exchanges.push(Arc::new(client));
    }
    */

    /* Bybit已被删除
    // Bybit
    if let (Ok(key), Ok(secret)) = (env::var("BYBIT_API_KEY"), env::var("BYBIT_SECRET")) {
        let testnet = env::var("BYBIT_TESTNET")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);
        let client = BybitClient::new(key, secret, testnet);
        exchanges.push(Arc::new(client));
    }
    */

    /* Gate已被删除
    // Gate
    if let (Ok(key), Ok(secret)) = (env::var("GATE_API_KEY"), env::var("GATE_SECRET")) {
        let testnet = env::var("GATE_TESTNET")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);
        let client = GateClient::new(key, secret, testnet);
        exchanges.push(Arc::new(client));
    }
    */

    // Hyperliquid
    if let (Ok(address), Ok(secret)) = (
        env::var("HYPERLIQUID_ADDRESS"),
        env::var("HYPERLIQUID_SECRET"),
    ) {
        let proxy_address =
            env::var("HYPERLIQUID_PROXY_ADDRESS").unwrap_or_else(|_| "".to_string());
        let testnet = env::var("HYPERLIQUID_TESTNET")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);
        let client = HyperliquidClient::new(address, proxy_address, secret, testnet);
        exchanges.push(Arc::new(client));
    }

    /* BSC Wallet已被删除
    // BSC Wallet
    if let (Ok(address), Ok(private_key)) = (env::var("BSC_ADDRESS"), env::var("BSC_PRIVATE_KEY")) {
        let testnet = env::var("BSC_TESTNET")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);
        let wallet = BscWallet::new(address, private_key, testnet);
        exchanges.push(Arc::new(wallet));
    }
    */

    /* Solana Wallet已被删除
    // Solana Wallet
    if let (Ok(address), Ok(private_key)) =
        (env::var("SOLANA_ADDRESS"), env::var("SOLANA_PRIVATE_KEY"))
    {
        let testnet = env::var("SOLANA_TESTNET")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);
        let wallet = SolanaWallet::new(address, private_key, testnet);
        exchanges.push(Arc::new(wallet));
    }
    */

    if exchanges.is_empty() {
        println!("❌ 未配置任何交易所 API");
        return Ok(());
    }

    println!("🏢 已加载 {} 个交易所\n", exchanges.len());

    // 汇总数据
    let mut total_balance = 0.0;
    let mut total_pnl = 0.0;
    let mut total_positions = 0;
    let mut total_margin_used = 0.0;

    // 遍历所有交易所
    for exchange in &exchanges {
        let name = exchange.get_exchange_name();

        println!("┌─────────────────────────────────────────────────────────────┐");
        println!(
            "│ 🏦 {}                                                    ",
            name
        );
        println!("├─────────────────────────────────────────────────────────────┤");

        // 获取账户信息
        match exchange.get_account_info().await {
            Ok(account) => {
                println!("│ 💰 账户余额");
                println!("│   总余额:        {:>15.2} USDT", account.total_balance);
                println!(
                    "│   可用余额:      {:>15.2} USDT",
                    account.available_balance
                );
                println!("│   未实现盈亏:    {:>15.2} USDT", account.unrealized_pnl);
                println!("│   已用保证金:    {:>15.2} USDT", account.margin_used);

                total_balance += account.total_balance;
                total_pnl += account.unrealized_pnl;
                total_margin_used += account.margin_used;
            }
            Err(e) => {
                println!("│ ❌ 获取账户信息失败: {}", e);
            }
        }

        println!("│");

        // 获取持仓信息
        match exchange.get_positions().await {
            Ok(positions) => {
                if positions.is_empty() {
                    println!("│ 📊 持仓: 无");
                } else {
                    println!("│ 📊 持仓 ({} 个)", positions.len());
                    for pos in &positions {
                        let pnl_icon = if pos.pnl >= 0.0 { "📈" } else { "📉" };
                        println!("│   {} {} {}", pnl_icon, pos.symbol, pos.side);
                        println!("│     数量:     {:>12.4}", pos.size);
                        println!("│     入场价:   {:>12.2} USDT", pos.entry_price);
                        println!("│     标记价:   {:>12.2} USDT", pos.mark_price);
                        println!("│     盈亏:     {:>12.2} USDT", pos.pnl);
                        println!("│     杠杆:     {:>12}x", pos.leverage);
                        println!("│     保证金:   {:>12.2} USDT", pos.margin);

                        let roi = if pos.margin > 0.0 {
                            (pos.pnl / pos.margin) * 100.0
                        } else {
                            0.0
                        };
                        println!("│     回报率:   {:>12.2}%", roi);
                        println!("│");
                    }
                    total_positions += positions.len();
                }
            }
            Err(e) => {
                println!("│ ❌ 获取持仓失败: {}", e);
            }
        }

        println!("└─────────────────────────────────────────────────────────────┘\n");
    }

    // 打印汇总
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║                        📊 总计汇总                            ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!(
        "║ 💎 总余额:           {:>15.2} USDT                    ║",
        total_balance
    );
    println!(
        "║ 💹 总未实现盈亏:     {:>15.2} USDT                    ║",
        total_pnl
    );
    println!(
        "║ 🔒 总已用保证金:     {:>15.2} USDT                    ║",
        total_margin_used
    );
    println!(
        "║ 📌 总持仓数:         {:>15} 个                       ║",
        total_positions
    );

    if total_balance > 0.0 {
        let total_roi = (total_pnl / total_balance) * 100.0;
        println!(
            "║ 📈 总回报率:         {:>15.2}%                       ║",
            total_roi
        );
    }

    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    // 如果有盈亏，显示排名
    if total_pnl != 0.0 {
        println!("📊 交易所盈亏排名:");
        println!("─────────────────────────────────────────────────────────────");

        let mut exchange_pnls: Vec<(String, f64)> = Vec::new();
        for exchange in &exchanges {
            if let Ok(account) = exchange.get_account_info().await {
                exchange_pnls.push((
                    exchange.get_exchange_name().to_string(),
                    account.unrealized_pnl,
                ));
            }
        }

        exchange_pnls.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        for (i, (name, pnl)) in exchange_pnls.iter().enumerate() {
            let icon = match i {
                0 => "🥇",
                1 => "🥈",
                2 => "🥉",
                _ => "  ",
            };
            let pnl_icon = if *pnl >= 0.0 { "📈" } else { "📉" };
            println!("{}  {} {}  {:>12.2} USDT", icon, pnl_icon, name, pnl);
        }
        println!();
    }

    Ok(())
}
