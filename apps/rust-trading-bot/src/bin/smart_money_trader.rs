use anyhow::Result;
use chrono::Local;
use log::{error, info, warn};
use rust_trading_bot::{
    binance_client::BinanceClient,
    deepseek_client::Kline,
    exchange_trait::ExchangeClient,
    gate_client::GateClient,
    okx_client::OkxClient,
    smart_money_tracker::{MoneyFlowDirection, MoneyFlowSignal, SmartMoneyTracker},
};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// 交易所类型
#[derive(Debug, Clone)]
enum ExchangeType {
    Binance,
    Okx,
    Gate,
}

/// 交易配置
struct TradingConfig {
    symbol: String,
    timeframe: String,
    leverage: u32,
    exchange: ExchangeType,
    base_position_usdt: f64,
    max_position_usdt: f64,
}

impl Default for TradingConfig {
    fn default() -> Self {
        Self {
            symbol: "BTC/USDT".to_string(),
            timeframe: "1h".to_string(),
            leverage: 5,
            exchange: ExchangeType::Gate,
            base_position_usdt: 50.0,
            max_position_usdt: 200.0,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    info!("╔══════════════════════════════════════════════════╗");
    info!("║     🎯 主力资金追踪交易系统 v1.0               ║");
    info!("╚══════════════════════════════════════════════════╝");
    info!("");

    let config = TradingConfig::default();

    info!("📊 交易配置:");
    info!("   交易对: {}", config.symbol);
    info!("   K线周期: {}", config.timeframe);
    info!("   杠杆倍数: {}x", config.leverage);
    info!("   交易所: {:?}", config.exchange);
    info!("   基础仓位: {} USDT", config.base_position_usdt);
    info!("   最大仓位: {} USDT", config.max_position_usdt);
    info!("");

    // 初始化交易所客户端
    match config.exchange {
        ExchangeType::Gate => {
            let api_key = std::env::var("GATE_API_KEY").expect("❌ 缺少 GATE_API_KEY 环境变量");
            let secret = std::env::var("GATE_SECRET").expect("❌ 缺少 GATE_SECRET 环境变量");

            let exchange = Arc::new(GateClient::new(api_key, secret, false));
            info!("✅ Gate.io 客户端初始化成功");

            run_trader(exchange, config).await?;
        }

        ExchangeType::Okx => {
            let api_key = std::env::var("OKX_API_KEY").expect("❌ 缺少 OKX_API_KEY 环境变量");
            let secret = std::env::var("OKX_SECRET").expect("❌ 缺少 OKX_SECRET 环境变量");
            let passphrase = std::env::var("OKX_PASSWORD").expect("❌ 缺少 OKX_PASSWORD 环境变量");

            let exchange = Arc::new(OkxClient::new(api_key, secret, passphrase, false));
            info!("✅ OKX 客户端初始化成功");

            run_trader(exchange, config).await?;
        }

        ExchangeType::Binance => {
            let api_key =
                std::env::var("BINANCE_API_KEY").expect("❌ 缺少 BINANCE_API_KEY 环境变量");
            let secret = std::env::var("BINANCE_SECRET").expect("❌ 缺少 BINANCE_SECRET 环境变量");

            let exchange = Arc::new(BinanceClient::new(api_key, secret, false));
            info!("✅ Binance 客户端初始化成功");

            run_trader(exchange, config).await?;
        }
    }

    Ok(())
}

async fn run_trader<T: ExchangeClient + 'static>(
    exchange: Arc<T>,
    config: TradingConfig,
) -> Result<()> {
    // 检查账户余额
    match ExchangeClient::get_account_info(&*exchange).await {
        Ok(account) => {
            info!("💰 账户余额: ${:.2} USDT", account.total_balance);
            info!("   可用余额: ${:.2} USDT", account.available_balance);
        }
        Err(e) => {
            error!("❌ 获取账户信息失败: {}", e);
        }
    }

    info!("");
    info!("🚀 主力资金追踪系统启动...");
    info!("");

    // 初始化追踪器
    let tracker = Arc::new(SmartMoneyTracker::new());

    // 示例：模拟接收主力资金信号
    info!("📡 等待主力资金信号...");
    info!("");
    info!("💡 提示：在实际使用中，你可以：");
    info!("   1. 通过 Telegram Bot 接收信号");
    info!("   2. 通过 Webhook API 接收信号");
    info!("   3. 手动触发交易信号");
    info!("");

    // 演示循环
    demo_trading_loop(&exchange, &tracker, &config).await?;

    Ok(())
}

/// 演示交易循环
async fn demo_trading_loop<T: ExchangeClient>(
    exchange: &Arc<T>,
    tracker: &Arc<SmartMoneyTracker>,
    config: &TradingConfig,
) -> Result<()> {
    loop {
        info!("═══════════════════════════════════════════");
        info!("📅 分析周期: {}", Local::now().format("%Y-%m-%d %H:%M:%S"));
        info!("═══════════════════════════════════════════");

        // 1. 获取 K 线数据（1小时）
        info!("📈 获取 1h K线数据...");
        let klines = match get_klines(exchange, &config.symbol).await {
            Ok(k) => k,
            Err(e) => {
                error!("❌ 获取K线失败: {}", e);
                sleep(Duration::from_secs(300)).await;
                continue;
            }
        };

        if klines.len() < 24 {
            warn!("⚠️  K线数据不足 (需要至少24根)");
            sleep(Duration::from_secs(300)).await;
            continue;
        }

        let current_price = klines.last().unwrap().close;
        info!("💰 当前价格: ${:.2}", current_price);

        // 2. 模拟接收主力资金信号
        // TODO: 实际使用时，这里应该从 Telegram/API 接收真实信号
        let demo_signal = create_demo_money_flow_signal();

        info!("📊 主力资金信号:");
        info!("   方向: {:?}", demo_signal.direction);
        info!("   强度: {:.2}", demo_signal.strength);
        info!("   来源: {}", demo_signal.source);
        info!("");

        // 3. 获取当前持仓
        let current_position = get_current_position(exchange, &config.symbol).await;

        // 4. 分析并生成交易信号
        match tracker.analyze_and_generate_signal(
            &demo_signal,
            &klines,
            current_price,
            current_position.as_deref(),
        ) {
            Some(signal) => {
                info!("{}", tracker.format_signal(&signal));

                // 5. 执行交易（演示模式，不实际下单）
                info!("🔔 检测到交易信号，但当前为演示模式，不执行实际交易");
                info!("");

                // TODO: 在实际使用中，取消下面的注释以执行真实交易
                // execute_trade(exchange, &signal, config).await?;
            }
            None => {
                info!("⏸️  暂无交易机会，继续观察...");
                info!("");
            }
        }

        // 等待下一个周期（1小时）
        info!("⏰ 等待下一个分析周期（1小时）...");
        info!("");
        sleep(Duration::from_secs(3600)).await;
    }
}

/// 获取K线数据
async fn get_klines<T: ExchangeClient>(exchange: &Arc<T>, symbol: &str) -> Result<Vec<Kline>> {
    let ohlcv = exchange.get_klines(symbol, "1h", Some(48)).await?;

    let klines: Vec<Kline> = ohlcv
        .iter()
        .map(|candle| Kline {
            timestamp: candle[0] as i64,
            open: candle[1],
            high: candle[2],
            low: candle[3],
            close: candle[4],
            volume: candle[5],
        })
        .collect();

    Ok(klines)
}

/// 获取当前持仓
async fn get_current_position<T: ExchangeClient>(
    exchange: &Arc<T>,
    symbol: &str,
) -> Option<String> {
    match exchange.get_positions().await {
        Ok(positions) => {
            for pos in positions {
                if pos.symbol.contains("BTC") && pos.size > 0.0 {
                    info!("📦 当前持仓:");
                    info!("   方向: {}", pos.side);
                    info!("   数量: {:.4}", pos.size);
                    info!("   入场价: ${:.2}", pos.entry_price);
                    info!("   盈亏: ${:.2}", pos.pnl);
                    info!("");
                    return Some(pos.side.clone());
                }
            }
            info!("📦 当前无持仓");
            info!("");
            None
        }
        Err(e) => {
            warn!("⚠️  获取持仓失败: {}", e);
            None
        }
    }
}

/// 创建演示用的主力资金信号
fn create_demo_money_flow_signal() -> MoneyFlowSignal {
    use chrono::Utc;

    // 这里是演示数据，实际使用时应该从外部信号源获取
    MoneyFlowSignal {
        timestamp: Utc::now().timestamp(),
        direction: MoneyFlowDirection::Inflow, // 模拟资金流入
        strength: 0.75,                        // 强度 75%
        source: "Demo".to_string(),
        symbol: "BTC/USDT".to_string(),
        note: Some("这是演示信号，实际使用请接入真实数据源".to_string()),
    }
}

/// 执行交易（实际交易逻辑）
#[allow(dead_code)]
async fn execute_trade<T: ExchangeClient>(
    exchange: &Arc<T>,
    signal: &rust_trading_bot::smart_money_tracker::TradingSignal,
    config: &TradingConfig,
) -> Result<()> {
    use rust_trading_bot::smart_money_tracker::{SignalPriority, SignalType};

    info!("🎯 执行交易信号...");

    // 根据信号优先级和类型执行交易
    match signal.signal_type {
        SignalType::LongBreakout | SignalType::LongPullback => {
            // 计算仓位
            let position_size = calculate_position_size(
                config.base_position_usdt,
                config.max_position_usdt,
                signal.confidence,
                &signal.priority,
            );

            info!("📊 开多仓:");
            info!("   数量: {:.2} USDT", position_size);
            info!("   入场: ${:.2}", signal.entry_price);
            info!("   止损: ${:.2}", signal.stop_loss);
            info!("   止盈: ${:.2}", signal.take_profit);

            // TODO: 实际下单
            // exchange.create_market_buy_order(...).await?;
        }

        SignalType::ShortBreakdown => {
            info!("📊 开空仓（破位做空）");
            // TODO: 实现做空逻辑
        }

        SignalType::ClosePosition => {
            info!("📊 平仓");
            // TODO: 实现平仓逻辑
        }

        SignalType::Hold => {
            info!("📊 持有现有仓位");
        }
    }

    Ok(())
}

/// 计算仓位大小
fn calculate_position_size(
    base_usdt: f64,
    max_usdt: f64,
    confidence: f64,
    priority: &rust_trading_bot::smart_money_tracker::SignalPriority,
) -> f64 {
    use rust_trading_bot::smart_money_tracker::SignalPriority;

    let priority_multiplier = match priority {
        SignalPriority::Critical => 1.5,
        SignalPriority::High => 1.2,
        SignalPriority::Medium => 1.0,
        SignalPriority::Low => 0.6,
    };

    let confidence_multiplier = confidence / 100.0;

    let position = base_usdt * priority_multiplier * confidence_multiplier;
    position.min(max_usdt)
}
