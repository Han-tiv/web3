use dotenv::dotenv;
use rust_trading_bot::binance_client::BinanceClient;
use std::collections::HashMap;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let api_key = env::var("BINANCE_API_KEY").expect("BINANCE_API_KEY not set");
    let secret_key = env::var("BINANCE_SECRET").expect("BINANCE_SECRET not set");

    let client = BinanceClient::new(api_key, secret_key, false);

    println!("🔍 正在获取最近12小时的交易数据...\n");

    // 获取收益历史
    let income_records = match client.get_income_history(12).await {
        Ok(records) => records,
        Err(e) => {
            eprintln!("❌ 获取收益历史失败: {}", e);
            return;
        }
    };

    // 获取成交记录
    let user_trades = match client.get_user_trades(12).await {
        Ok(trades) => trades,
        Err(e) => {
            eprintln!("❌ 获取成交记录失败: {}", e);
            return;
        }
    };

    if income_records.is_empty() {
        println!("⚠️  最近12小时没有交易记录");
        return;
    }

    println!(
        "📊 获取到 {} 条收益记录, {} 条成交记录\n",
        income_records.len(),
        user_trades.len()
    );
    println!("{:=<100}", "");

    // 按币种统计保证金使用
    let mut symbol_margin: HashMap<String, f64> = HashMap::new();
    const DEFAULT_LEVERAGE: f64 = 10.0;

    for trade in &user_trades {
        let notional = trade.quoteQty.parse::<f64>().unwrap_or(0.0);

        // 判断是否为开仓单
        let is_open_trade = (trade.side == "BUY" && trade.positionSide == "LONG")
            || (trade.side == "SELL" && trade.positionSide == "SHORT");

        if is_open_trade && notional > 0.0 {
            let margin = notional / DEFAULT_LEVERAGE;
            *symbol_margin.entry(trade.symbol.clone()).or_insert(0.0) += margin;
        }
    }

    // 按币种统计收益
    let mut symbol_stats: HashMap<String, SymbolPnl> = HashMap::new();

    for record in &income_records {
        let income: f64 = record.income.parse().unwrap_or(0.0);

        let stats = symbol_stats
            .entry(record.symbol.clone())
            .or_insert(SymbolPnl {
                symbol: record.symbol.clone(),
                total_pnl: 0.0,
                trade_count: 0,
                win_count: 0,
                loss_count: 0,
                max_profit: 0.0,
                max_loss: 0.0,
                total_margin: 0.0,
                margin_loss_rate: 0.0,
            });

        stats.total_pnl += income;
        stats.trade_count += 1;

        if income > 0.0 {
            stats.win_count += 1;
            if income > stats.max_profit {
                stats.max_profit = income;
            }
        } else if income < 0.0 {
            stats.loss_count += 1;
            if income < stats.max_loss {
                stats.max_loss = income;
            }
        }
    }

    // 合并保证金数据
    for (symbol, margin) in symbol_margin {
        if let Some(stats) = symbol_stats.get_mut(&symbol) {
            stats.total_margin = margin;
            if margin > 0.0 {
                stats.margin_loss_rate = (stats.total_pnl / margin) * 100.0;
            }
        }
    }

    // 排序：按总盈亏排序，亏损的排前面
    let mut stats_vec: Vec<_> = symbol_stats.into_iter().map(|(_, v)| v).collect();
    stats_vec.sort_by(|a, b| a.total_pnl.partial_cmp(&b.total_pnl).unwrap());

    println!("📈 币种收益统计 (最近12小时):\n");

    for stat in &stats_vec {
        let win_rate = if stat.trade_count > 0 {
            (stat.win_count as f64 / stat.trade_count as f64) * 100.0
        } else {
            0.0
        };

        let emoji = if stat.total_pnl > 0.0 { "✅" } else { "❌" };
        let avg_pnl = stat.total_pnl / stat.trade_count as f64;

        println!("{} {}", emoji, stat.symbol);
        println!(
            "   交易次数: {} 笔 ({}胜 {}负), 胜率: {:.1}%",
            stat.trade_count, stat.win_count, stat.loss_count, win_rate
        );
        println!(
            "   总盈亏: {:.4} USDT (平均每笔: {:.4} USDT)",
            stat.total_pnl, avg_pnl
        );

        if stat.total_margin > 0.0 {
            println!(
                "   投入保证金: {:.2} USDT ({}x杠杆估算)",
                stat.total_margin, DEFAULT_LEVERAGE
            );
            println!("   保证金收益率: {:.2}%", stat.margin_loss_rate);
        }

        println!(
            "   最大盈利: {:.4} USDT | 最大亏损: {:.4} USDT",
            stat.max_profit, stat.max_loss
        );
        println!();
    }

    println!("{:=<100}", "");

    // 识别高风险币种
    println!("\n⚠️  风险等级评估:\n");

    let high_risk: Vec<_> = stats_vec
        .iter()
        .filter(|s| s.total_margin > 0.0 && s.margin_loss_rate < -15.0)
        .collect();

    let medium_risk: Vec<_> = stats_vec
        .iter()
        .filter(|s| {
            s.total_margin > 0.0 && s.margin_loss_rate >= -15.0 && s.margin_loss_rate < -10.0
        })
        .collect();

    if !high_risk.is_empty() {
        println!("🔴 高风险币种 (保证金亏损率 > 15%):");
        for stat in high_risk {
            println!(
                "   {} - 亏损率 {:.2}%, 总亏损 {:.4} USDT, {}胜{}负",
                stat.symbol, stat.margin_loss_rate, stat.total_pnl, stat.win_count, stat.loss_count
            );
        }
        println!();
    }

    if !medium_risk.is_empty() {
        println!("🟡 中风险币种 (保证金亏损率 10-15%):");
        for stat in medium_risk {
            println!(
                "   {} - 亏损率 {:.2}%, 总亏损 {:.4} USDT, {}胜{}负",
                stat.symbol, stat.margin_loss_rate, stat.total_pnl, stat.win_count, stat.loss_count
            );
        }
        println!();
    }

    let profitable: Vec<_> = stats_vec
        .iter()
        .filter(|s| s.total_pnl > 0.5 && s.total_margin > 0.0)
        .collect();

    if !profitable.is_empty() {
        println!("🟢 优秀币种 (盈利 > 0.5 USDT):");
        for stat in profitable {
            println!(
                "   {} - 收益率 {:.2}%, 总盈利 {:.4} USDT, {}胜{}负",
                stat.symbol, stat.margin_loss_rate, stat.total_pnl, stat.win_count, stat.loss_count
            );
        }
        println!();
    }

    // 总结
    let total_pnl: f64 = stats_vec.iter().map(|s| s.total_pnl).sum();
    let total_trades: usize = stats_vec.iter().map(|s| s.trade_count).sum();
    let total_margin: f64 = stats_vec.iter().map(|s| s.total_margin).sum();
    let overall_rate = if total_margin > 0.0 {
        (total_pnl / total_margin) * 100.0
    } else {
        0.0
    };

    println!("{:=<100}", "");
    println!("\n📊 总体统计:");
    println!("   总交易次数: {} 笔", total_trades);
    println!("   总盈亏: {:.4} USDT", total_pnl);
    println!(
        "   总保证金投入: {:.2} USDT ({}x杠杆估算)",
        total_margin, DEFAULT_LEVERAGE
    );
    println!("   总体保证金收益率: {:.2}%", overall_rate);
    println!("   币种数: {}", stats_vec.len());

    println!(
        "\n💡 说明: 保证金基于成交记录和{}x平均杠杆估算,误差约±20%",
        DEFAULT_LEVERAGE
    );
}

#[derive(Debug)]
struct SymbolPnl {
    symbol: String,
    total_pnl: f64,
    trade_count: usize,
    win_count: usize,
    loss_count: usize,
    max_profit: f64,
    max_loss: f64,
    total_margin: f64,
    margin_loss_rate: f64,
}
