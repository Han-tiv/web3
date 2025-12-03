use anyhow::{anyhow, Result};
use chrono::{Local, Timelike};
use log::{error, info, warn};
use rust_trading_bot::{
    binance_client::BinanceClient,
    deepseek_client::{DeepSeekClient, Kline, Position},
    exchange_trait::{ExchangeClient, ExchangeType},
    gate_client::GateClient,
    okx_client::OkxClient,
    technical_analysis::TechnicalAnalyzer,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

// DeepSeek AI 交易仅支持 Binance

// 支持的交易币种
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
enum TradingSymbol {
    BTC,
    ETH,
    SOL,
    BNB,
    DOGE,
    XRP,
}

impl TradingSymbol {
    fn to_pair(&self) -> String {
        match self {
            TradingSymbol::BTC => "BTC/USDT".to_string(),
            TradingSymbol::ETH => "ETH/USDT".to_string(),
            TradingSymbol::SOL => "SOL/USDT".to_string(),
            TradingSymbol::BNB => "BNB/USDT".to_string(),
            TradingSymbol::DOGE => "DOGE/USDT".to_string(),
            TradingSymbol::XRP => "XRP/USDT".to_string(),
        }
    }

    fn get_min_amount(&self) -> f64 {
        match self {
            TradingSymbol::BTC => 0.0001, // 最小 0.0001 BTC
            TradingSymbol::ETH => 0.001,  // 最小 0.001 ETH
            TradingSymbol::SOL => 0.01,   // 最小 0.01 SOL
            TradingSymbol::BNB => 0.01,   // 最小 0.01 BNB
            TradingSymbol::DOGE => 1.0,   // 最小 1 DOGE
            TradingSymbol::XRP => 1.0,    // 最小 1 XRP
        }
    }

    fn get_display_name(&self) -> &str {
        match self {
            TradingSymbol::BTC => "Bitcoin",
            TradingSymbol::ETH => "Ethereum",
            TradingSymbol::SOL => "Solana",
            TradingSymbol::BNB => "Binance Coin",
            TradingSymbol::DOGE => "Dogecoin",
            TradingSymbol::XRP => "Ripple",
        }
    }

    fn from_string(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "BTC" | "BITCOIN" => Some(TradingSymbol::BTC),
            "ETH" | "ETHEREUM" => Some(TradingSymbol::ETH),
            "SOL" | "SOLANA" => Some(TradingSymbol::SOL),
            "BNB" | "BINANCE" => Some(TradingSymbol::BNB),
            "DOGE" | "DOGECOIN" => Some(TradingSymbol::DOGE),
            "XRP" | "RIPPLE" => Some(TradingSymbol::XRP),
            _ => None,
        }
    }

    fn all_symbols() -> Vec<TradingSymbol> {
        vec![
            TradingSymbol::BTC,
            TradingSymbol::ETH,
            TradingSymbol::SOL,
            TradingSymbol::BNB,
            TradingSymbol::DOGE,
            TradingSymbol::XRP,
        ]
    }
}

// 智能仓位配置
#[derive(Debug, Clone)]
struct PositionConfig {
    base_usdt: f64,                    // 基础投入金额
    high_confidence_multiplier: f64,   // 高信心倍数 1.5x
    medium_confidence_multiplier: f64, // 中信心倍数 1.0x
    low_confidence_multiplier: f64,    // 低信心倍数 0.5x
    max_position_ratio: f64,           // 最大仓位比例 10%
    trend_strength_multiplier: f64,    // 趋势强度倍数 1.2x
}

impl Default for PositionConfig {
    fn default() -> Self {
        Self {
            base_usdt: 6.0, // 降低到 6 USDT，每次开单 3-11 USDT
            high_confidence_multiplier: 1.5,
            medium_confidence_multiplier: 1.0,
            low_confidence_multiplier: 0.5,
            max_position_ratio: 0.10, // 最大 10% = 10 USDT (100U账户)
            trend_strength_multiplier: 1.2,
        }
    }
}

// 交易信号记录
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignalRecord {
    timestamp: String,
    signal: String,
    confidence: String,
    reason: String,
    price: f64,
}

// 信号历史管理
struct SignalHistory {
    signals: VecDeque<SignalRecord>,
    max_size: usize,
}

impl SignalHistory {
    fn new(max_size: usize) -> Self {
        Self {
            signals: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    fn add(&mut self, record: SignalRecord) {
        if self.signals.len() >= self.max_size {
            self.signals.pop_front();
        }
        self.signals.push_back(record);
    }

    fn get_recent(&self, count: usize) -> Vec<&SignalRecord> {
        self.signals.iter().rev().take(count).collect()
    }

    fn count_signal(&self, signal: &str, last_n: usize) -> usize {
        self.signals
            .iter()
            .rev()
            .take(last_n)
            .filter(|s| s.signal == signal)
            .count()
    }
}

// 交易配置
struct TradingConfig {
    trading_symbol: TradingSymbol,
    symbol: String,
    timeframe: String,
    amount: f64,
    leverage: u32,
    interval_minutes: u64,
    exchange: ExchangeType,
    position_config: PositionConfig,
}

impl TradingConfig {
    fn new(trading_symbol: TradingSymbol) -> Self {
        let symbol = trading_symbol.to_pair();
        let amount = trading_symbol.get_min_amount();

        Self {
            trading_symbol,
            symbol,
            timeframe: "15m".to_string(),
            amount, // 使用币种对应的最小值
            leverage: 5,
            interval_minutes: 15,
            exchange: ExchangeType::Gate,
            position_config: PositionConfig::default(),
        }
    }
}

impl Default for TradingConfig {
    fn default() -> Self {
        Self::new(TradingSymbol::BTC)
    }
}

// 等待到下一个整点
fn wait_for_next_period(interval_minutes: u64) -> Duration {
    let now = Local::now();
    let current_minute = now.minute() as u64;
    let current_second = now.second() as u64;

    // 计算下一个整点时间
    let next_period_minute = ((current_minute / interval_minutes) + 1) * interval_minutes;

    let minutes_to_wait = if next_period_minute >= 60 {
        60 - current_minute + (next_period_minute - 60)
    } else {
        next_period_minute - current_minute
    };

    let seconds_to_wait = (minutes_to_wait * 60) - current_second;

    // 显示友好的等待时间
    let display_minutes = if current_second > 0 {
        minutes_to_wait.saturating_sub(1)
    } else {
        minutes_to_wait
    };
    let display_seconds = if current_second > 0 {
        60 - current_second
    } else {
        0
    };

    if display_minutes > 0 {
        info!(
            "🕒 等待 {} 分 {} 秒到整点...",
            display_minutes, display_seconds
        );
    } else {
        info!("🕒 等待 {} 秒到整点...", display_seconds);
    }

    Duration::from_secs(seconds_to_wait)
}

// 计算智能仓位
async fn calculate_intelligent_position<T: ExchangeClient>(
    exchange: &Arc<T>,
    signal_confidence: &str,
    price: f64,
    rsi: f64,
    trend: &str,
    config: &PositionConfig,
    trading_config: &TradingConfig,
) -> Result<f64> {
    // 获取账户余额
    let account = exchange.get_account_info().await?;
    let usdt_balance = account.available_balance;

    info!(
        "💰 可用USDT余额: {:.2}, 下单基数: {:.2}",
        usdt_balance, config.base_usdt
    );

    // 根据信心程度调整
    let confidence_multiplier = match signal_confidence {
        "HIGH" => config.high_confidence_multiplier,
        "MEDIUM" => config.medium_confidence_multiplier,
        "LOW" => config.low_confidence_multiplier,
        _ => 1.0,
    };

    // 根据趋势强度调整
    let trend_multiplier = if trend.contains("强势") {
        config.trend_strength_multiplier
    } else {
        1.0
    };

    // 根据RSI状态调整（超买超卖区域减仓）
    let rsi_multiplier = if !(25.0..=75.0).contains(&rsi) {
        0.7
    } else {
        1.0
    };

    // 计算建议投入USDT金额
    let suggested_usdt =
        config.base_usdt * confidence_multiplier * trend_multiplier * rsi_multiplier;

    // 风险管理：不超过总资金的指定比例
    let max_usdt = usdt_balance * config.max_position_ratio;
    let final_usdt = suggested_usdt.min(max_usdt);

    // 计算币种数量
    let coin_amount = final_usdt / price;

    let symbol_name = format!("{:?}", trading_config.trading_symbol);

    info!("📊 仓位计算详情:");
    info!("   - 基础USDT: {:.2}", config.base_usdt);
    info!("   - 信心倍数: {:.2}", confidence_multiplier);
    info!("   - 趋势倍数: {:.2}", trend_multiplier);
    info!("   - RSI倍数: {:.2}", rsi_multiplier);
    info!("   - 建议USDT: {:.2}", suggested_usdt);
    info!("   - 最终USDT: {:.2}", final_usdt);
    info!("   - {}数量: {:.6}", symbol_name, coin_amount);

    // 确保最小交易量
    let min_amount = trading_config.trading_symbol.get_min_amount();
    let final_amount = coin_amount.max(min_amount);

    if final_amount > coin_amount {
        info!(
            "   ⚠️  调整到最小交易量: {:.6} {}",
            final_amount, symbol_name
        );
    }

    info!(
        "🎯 最终仓位: {:.2} USDT → {:.6} {}",
        final_usdt, final_amount, symbol_name
    );

    Ok(final_amount)
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    // 加载环境变量
    dotenv::dotenv().ok();

    info!("═══════════════════════════════════════════");
    info!("🤖 DeepSeek AI Trading Bot v3.0 - Multi-Coin");
    info!("═══════════════════════════════════════════");
    info!("");

    // 显示支持的币种
    info!("💎 支持的交易币种:");
    for symbol in TradingSymbol::all_symbols() {
        info!("   - {} ({})", symbol.get_display_name(), symbol.to_pair());
    }
    info!("");

    // 从环境变量读取币种配置
    let trading_symbol = std::env::var("TRADING_SYMBOL")
        .ok()
        .and_then(|s| TradingSymbol::from_string(&s))
        .unwrap_or_else(|| {
            info!("💡 未设置 TRADING_SYMBOL 环境变量，使用默认币种: BTC");
            info!("   提示: 设置 TRADING_SYMBOL=ETH 可以交易以太坊");
            info!("");
            TradingSymbol::BTC
        });

    // 加载配置
    let config = TradingConfig::new(trading_symbol);

    info!(
        "✅ 当前选择: {} ({})",
        config.trading_symbol.get_display_name(),
        config.symbol
    );
    info!("");

    // 初始化 Binance 客户端
    let deepseek_key =
        std::env::var("DEEPSEEK_API_KEY").expect("❌ 缺少 DEEPSEEK_API_KEY 环境变量");
    let deepseek = Arc::new(DeepSeekClient::new(deepseek_key));

    let analyzer = Arc::new(TechnicalAnalyzer::new());

    info!("📊 交易配置:");
    info!("   币种: {}", config.trading_symbol.get_display_name());
    info!("   交易对: {}", config.symbol);
    info!("   K线周期: {}", config.timeframe);
    info!(
        "   最小交易量: {} {:?}",
        config.amount, config.trading_symbol
    );
    info!("   杠杆倍数: {}x", config.leverage);
    info!("   执行间隔: {} 分钟", config.interval_minutes);
    info!("   交易所: {:?}", config.exchange);
    info!("");

    // 初始化交易所客户端（根据配置选择）
    match config.exchange {
        ExchangeType::Binance => {
            let api_key =
                std::env::var("BINANCE_API_KEY").expect("❌ 缺少 BINANCE_API_KEY 环境变量");
            let secret = std::env::var("BINANCE_SECRET").expect("❌ 缺少 BINANCE_SECRET 环境变量");

            let exchange = Arc::new(BinanceClient::new(api_key, secret, false));
            info!("✅ Binance 客户端初始化成功");

            run_bot(exchange, deepseek, analyzer, config).await?;
        }

        ExchangeType::Okx => {
            let api_key = std::env::var("OKX_API_KEY").expect("❌ 缺少 OKX_API_KEY 环境变量");
            let secret = std::env::var("OKX_SECRET").expect("❌ 缺少 OKX_SECRET 环境变量");
            let passphrase = std::env::var("OKX_PASSWORD").expect("❌ 缺少 OKX_PASSWORD 环境变量");

            let exchange = Arc::new(OkxClient::new(api_key, secret, passphrase, false));
            info!("✅ OKX 客户端初始化成功");

            run_bot(exchange, deepseek, analyzer, config).await?;
        }

        ExchangeType::Gate => {
            let api_key = std::env::var("GATE_API_KEY").expect("❌ 缺少 GATE_API_KEY 环境变量");
            let secret = std::env::var("GATE_SECRET").expect("❌ 缺少 GATE_SECRET 环境变量");

            let exchange = Arc::new(GateClient::new(api_key, secret, false));
            info!("✅ Gate.io 客户端初始化成功");

            run_bot(exchange, deepseek, analyzer, config).await?;
        }

        ExchangeType::Bitget | ExchangeType::Bybit => {
            return Err(anyhow!(
                "当前 DeepSeek 交易器尚未支持 {:?} 交易所",
                config.exchange
            ));
        }
    }

    Ok(())
}

// 运行交易机器人的主循环
async fn run_bot<T: ExchangeClient + 'static>(
    exchange: Arc<T>,
    deepseek: Arc<DeepSeekClient>,
    analyzer: Arc<TechnicalAnalyzer>,
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
    info!("🚀 开始运行交易机器人...");
    info!("📍 执行模式: 每 {} 分钟整点执行", config.interval_minutes);
    info!("");

    // 创建信号历史管理器
    let mut signal_history = SignalHistory::new(30);

    // 首次等待到整点
    let initial_wait = wait_for_next_period(config.interval_minutes);
    sleep(initial_wait).await;

    // 主循环
    loop {
        let cycle_start = Local::now();
        info!("═══════════════════════════════════════════");
        info!("📅 交易周期: {}", cycle_start.format("%Y-%m-%d %H:%M:%S"));
        info!("═══════════════════════════════════════════");

        match run_trading_cycle(
            &exchange,
            &deepseek,
            &analyzer,
            &config,
            &mut signal_history,
        )
        .await
        {
            Ok(_) => info!("✅ 交易周期完成"),
            Err(e) => error!("❌ 交易周期错误: {}", e),
        }

        let cycle_duration = Local::now().signed_duration_since(cycle_start);
        info!("⏱️  周期用时: {} 秒", cycle_duration.num_seconds());
        info!("");

        // 等待到下一个整点
        let wait_time = wait_for_next_period(config.interval_minutes);
        sleep(wait_time).await;
    }
}

async fn run_trading_cycle<T: ExchangeClient>(
    exchange: &Arc<T>,
    deepseek: &Arc<DeepSeekClient>,
    analyzer: &Arc<TechnicalAnalyzer>,
    config: &TradingConfig,
    signal_history: &mut SignalHistory,
) -> Result<()> {
    // 1. 获取 K 线数据
    info!("📈 获取 K 线数据...");
    let klines = get_klines(exchange, &config.symbol).await?;

    if klines.len() < 50 {
        warn!("⚠️  K 线数据不足 (需要至少 50 根)，本周期跳过");
        return Ok(());
    }

    let current_price = klines.last().unwrap().close;
    info!("💰 当前价格: ${:.2}", current_price);

    // 2. 计算技术指标
    info!("🔢 计算技术指标...");
    let indicators = analyzer.calculate_indicators(&klines);

    // 获取趋势判断
    let trend = analyzer.determine_trend(&indicators, current_price);
    let rsi_signal = analyzer.get_rsi_signal(indicators.rsi);
    let bb_signal = analyzer.get_bollinger_signal(
        current_price,
        indicators.bb_upper,
        indicators.bb_lower,
        indicators.bb_middle,
    );

    info!("   趋势: {}", trend);
    info!("   RSI: {:.2} ({})", indicators.rsi, rsi_signal);
    info!("   布林带: {}", bb_signal);

    // 3. 获取当前持仓
    info!("📦 查询持仓...");
    let positions = exchange.get_positions().await?;
    let current_position = positions
        .iter()
        .find(|p| p.symbol.contains("BTC") && p.size > 0.0)
        .map(|p| Position {
            side: p.side.clone(),
            size: p.size,
            entry_price: p.entry_price,
            unrealized_pnl: p.pnl,
        });

    if let Some(ref pos) = current_position {
        info!("   持仓方向: {}", pos.side);
        info!("   持仓数量: {:.4} BTC", pos.size);
        info!("   入场价格: ${:.2}", pos.entry_price);
        info!("   未实现盈亏: ${:.2}", pos.unrealized_pnl);
    } else {
        info!("   当前无持仓");
    }

    // 4. 构建 prompt 并调用 DeepSeek
    info!("🧠 AI 分析中...");
    let prompt = deepseek.build_prompt(
        &klines,
        &indicators,
        current_price,
        current_position.as_ref(),
    );

    let signal = match deepseek.analyze_market(&prompt).await {
        Ok(s) => s,
        Err(e) => {
            error!("❌ DeepSeek 分析失败: {}", e);
            return Ok(());
        }
    };

    info!("📡 AI 分析结果:");
    info!("   信号: {}", signal.signal);
    info!("   置信度: {}", signal.confidence);
    info!("   理由: {}", signal.reason);
    info!("   止损价: ${:.2}", signal.stop_loss.unwrap_or(0.0));
    if signal.stop_loss.is_none() {
        info!("   ⚠️  AI未提供止损价");
    }
    info!("   止盈价: ${:.2}", signal.take_profit.unwrap_or(0.0));
    if signal.take_profit.is_none() {
        info!("   📌 采用动态止盈策略(由AI监控持仓管理)");
    }

    // 5. 记录信号到历史
    let signal_record = SignalRecord {
        timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        signal: signal.signal.clone(),
        confidence: signal.confidence.clone(),
        reason: signal.reason.clone(),
        price: current_price,
    };
    signal_history.add(signal_record);

    // 显示信号统计
    let buy_count = signal_history.count_signal("BUY", 10);
    let sell_count = signal_history.count_signal("SELL", 10);
    let hold_count = signal_history.count_signal("HOLD", 10);
    info!(
        "📊 最近10次信号: BUY({}) SELL({}) HOLD({})",
        buy_count, sell_count, hold_count
    );

    // 6. 防频繁交易检查
    let should_skip = check_frequent_trading(&signal, current_position.as_ref(), signal_history);
    if should_skip {
        info!("🔒 防频繁交易：本周期跳过执行");
        return Ok(());
    }

    // 8. 计算智能仓位
    let position_size = if signal.signal != "HOLD" {
        calculate_intelligent_position(
            exchange,
            &signal.confidence,
            current_price,
            indicators.rsi,
            &trend,
            &config.position_config,
            config,
        )
        .await?
    } else {
        config.amount
    };

    // 9. 执行交易决策
    info!("🎯 执行交易决策...");
    execute_trading_decision(
        exchange,
        &signal,
        current_position.as_ref(),
        config,
        current_price,
        position_size,
    )
    .await?;

    Ok(())
}

// 检查是否应该因频繁交易而跳过
fn check_frequent_trading(
    signal: &rust_trading_bot::deepseek_client::TradingSignal,
    current_position: Option<&Position>,
    signal_history: &SignalHistory,
) -> bool {
    // 如果是 HOLD 信号，直接返回
    if signal.signal == "HOLD" {
        return false;
    }

    // 如果当前有持仓，检查是否反向信号
    if let Some(pos) = current_position {
        let is_reverse_signal = (pos.side == "long" && signal.signal == "SELL")
            || (pos.side == "short" && signal.signal == "BUY");

        if is_reverse_signal {
            // 反向信号需要高信心才执行
            if signal.confidence != "HIGH" {
                info!(
                    "   当前持仓: {} | 信号: {} | 信心: {}",
                    pos.side, signal.signal, signal.confidence
                );
                info!("   ⚠️  非高信心反向信号，保持现有仓位");
                return true;
            }

            // 检查最近是否已经出现过相同信号
            let recent_signals = signal_history.get_recent(3);
            let same_signal_count = recent_signals
                .iter()
                .filter(|s| s.signal == signal.signal)
                .count();

            if same_signal_count >= 2 {
                info!(
                    "   ⚠️  最近3次中已出现{}次{}信号，避免频繁反转",
                    same_signal_count, signal.signal
                );
                return true;
            }
        }
    }

    false
}

async fn get_klines<T: ExchangeClient>(exchange: &Arc<T>, symbol: &str) -> Result<Vec<Kline>> {
    // 获取最近 100 根 K 线 (15分钟 = 25小时数据)
    let price = exchange.get_current_price(symbol).await?;

    // 简化版：生成模拟 K 线数据
    // 实际应该调用交易所 API 获取历史数据
    let mut klines = Vec::new();
    let base_price = price;

    for i in 0..100 {
        let volatility = 0.002; // 0.2% 波动
        let open = base_price * (1.0 + (i as f64 * 0.0001 - 0.005));
        let close = open * (1.0 + (rand::random() - 0.5) * volatility);
        let high = open.max(close) * (1.0 + rand::random() * volatility);
        let low = open.min(close) * (1.0 - rand::random() * volatility);

        let volume = 10.0 + rand::random() * 5.0;
        let quote_volume = volume * close;
        let taker_buy_volume = volume * (0.4 + rand::random() * 0.4);
        let taker_buy_quote_volume = taker_buy_volume * close;

        klines.push(Kline {
            timestamp: (i as i64) * 900000, // 15分钟
            open,
            high,
            low,
            close,
            volume,
            quote_volume,
            taker_buy_volume,
            taker_buy_quote_volume,
        });
    }

    Ok(klines)
}

async fn execute_trading_decision<T: ExchangeClient>(
    exchange: &Arc<T>,
    signal: &rust_trading_bot::deepseek_client::TradingSignal,
    current_position: Option<&Position>,
    config: &TradingConfig,
    current_price: f64,
    position_size: f64,
) -> Result<()> {
    // 低信心信号跳过执行
    if signal.confidence == "LOW" {
        info!("⚠️  低信心信号，跳过执行");
        return Ok(());
    }

    match signal.signal.as_str() {
        "BUY" => {
            match current_position {
                None => {
                    // 无持仓，开多仓
                    info!("🟢 开多仓");
                    info!("   交易对: {}", config.symbol);
                    info!("   数量: {:.6} BTC", position_size);
                    info!("   价格: ${:.2}", current_price);
                    info!("   杠杆: {}x", config.leverage);

                    match exchange
                        .open_long(
                            &config.symbol,
                            position_size,
                            config.leverage,
                            "cross",
                            false,
                        )
                        .await
                    {
                        Ok(_) => {
                            info!("✅ 开多仓成功！");
                            info!("   止损价: ${:.2}", signal.stop_loss.unwrap_or(0.0));
                            if signal.stop_loss.is_none() {
                                info!("   ⚠️  AI未提供止损价");
                            }
                            info!("   止盈价: ${:.2}", signal.take_profit.unwrap_or(0.0));
                            if signal.take_profit.is_none() {
                                info!("   📌 采用动态止盈策略(由AI监控持仓管理)");
                            }
                        }
                        Err(e) => error!("❌ 开多仓失败: {}", e),
                    }
                }
                Some(pos) if pos.side == "long" => {
                    // 已有多头持仓，检查是否需要加仓/减仓
                    let size_diff = position_size - pos.size;
                    let size_diff_abs = size_diff.abs();

                    if size_diff_abs >= 0.0001 {
                        // 有显著差异
                        if size_diff > 0.0 {
                            // 加仓
                            let add_size = size_diff;
                            info!(
                                "📈 多仓加仓 {:.6} BTC (当前:{:.6} → 目标:{:.6})",
                                add_size, pos.size, position_size
                            );

                            match exchange
                                .open_long(
                                    &config.symbol,
                                    add_size,
                                    config.leverage,
                                    "cross",
                                    false,
                                )
                                .await
                            {
                                Ok(_) => info!("✅ 加仓成功"),
                                Err(e) => error!("❌ 加仓失败: {}", e),
                            }
                        } else {
                            // 减仓
                            let reduce_size = size_diff_abs;
                            info!(
                                "📉 多仓减仓 {:.6} BTC (当前:{:.6} → 目标:{:.6})",
                                reduce_size, pos.size, position_size
                            );

                            match exchange
                                .close_position(&config.symbol, "long", reduce_size)
                                .await
                            {
                                Ok(_) => info!("✅ 减仓成功"),
                                Err(e) => error!("❌ 减仓失败: {}", e),
                            }
                        }
                    } else {
                        info!(
                            "⏸️  多仓仓位合适，保持现状 (当前:{:.6}, 目标:{:.6})",
                            pos.size, position_size
                        );
                    }
                }
                Some(pos) if pos.side == "short" => {
                    // 有空头持仓，先平空再开多
                    info!(
                        "🔄 平空仓 {:.6} BTC 并开多仓 {:.6} BTC",
                        pos.size, position_size
                    );

                    // 平空仓
                    match exchange
                        .close_position(&config.symbol, "short", pos.size)
                        .await
                    {
                        Ok(_) => {
                            info!("✅ 平空仓成功");
                            sleep(Duration::from_secs(1)).await;

                            // 开多仓
                            match exchange
                                .open_long(
                                    &config.symbol,
                                    position_size,
                                    config.leverage,
                                    "cross",
                                    false,
                                )
                                .await
                            {
                                Ok(_) => info!("✅ 开多仓成功"),
                                Err(e) => error!("❌ 开多仓失败: {}", e),
                            }
                        }
                        Err(e) => error!("❌ 平空仓失败: {}", e),
                    }
                }
                _ => {}
            }
        }

        "SELL" => {
            match current_position {
                None => {
                    // 无持仓，开空仓
                    info!("🔴 开空仓");
                    info!("   交易对: {}", config.symbol);
                    info!("   数量: {:.6} BTC", position_size);
                    info!("   价格: ${:.2}", current_price);
                    info!("   杠杆: {}x", config.leverage);

                    match exchange
                        .open_short(
                            &config.symbol,
                            position_size,
                            config.leverage,
                            "cross",
                            false,
                        )
                        .await
                    {
                        Ok(_) => {
                            info!("✅ 开空仓成功！");
                            info!("   止损价: ${:.2}", signal.stop_loss.unwrap_or(0.0));
                            if signal.stop_loss.is_none() {
                                info!("   ⚠️  AI未提供止损价");
                            }
                            info!("   止盈价: ${:.2}", signal.take_profit.unwrap_or(0.0));
                            if signal.take_profit.is_none() {
                                info!("   📌 采用动态止盈策略(由AI监控持仓管理)");
                            }
                        }
                        Err(e) => error!("❌ 开空仓失败: {}", e),
                    }
                }
                Some(pos) if pos.side == "short" => {
                    // 已有空头持仓，检查是否需要加仓/减仓
                    let size_diff = position_size - pos.size;
                    let size_diff_abs = size_diff.abs();

                    if size_diff_abs >= 0.0001 {
                        if size_diff > 0.0 {
                            // 加仓
                            let add_size = size_diff;
                            info!(
                                "📈 空仓加仓 {:.6} BTC (当前:{:.6} → 目标:{:.6})",
                                add_size, pos.size, position_size
                            );

                            match exchange
                                .open_short(
                                    &config.symbol,
                                    add_size,
                                    config.leverage,
                                    "cross",
                                    false,
                                )
                                .await
                            {
                                Ok(_) => info!("✅ 加仓成功"),
                                Err(e) => error!("❌ 加仓失败: {}", e),
                            }
                        } else {
                            // 减仓
                            let reduce_size = size_diff_abs;
                            info!(
                                "📉 空仓减仓 {:.6} BTC (当前:{:.6} → 目标:{:.6})",
                                reduce_size, pos.size, position_size
                            );

                            match exchange
                                .close_position(&config.symbol, "short", reduce_size)
                                .await
                            {
                                Ok(_) => info!("✅ 减仓成功"),
                                Err(e) => error!("❌ 减仓失败: {}", e),
                            }
                        }
                    } else {
                        info!(
                            "⏸️  空仓仓位合适，保持现状 (当前:{:.6}, 目标:{:.6})",
                            pos.size, position_size
                        );
                    }
                }
                Some(pos) if pos.side == "long" => {
                    // 有多头持仓，先平多再开空
                    info!(
                        "🔄 平多仓 {:.6} BTC 并开空仓 {:.6} BTC",
                        pos.size, position_size
                    );

                    // 平多仓
                    match exchange
                        .close_position(&config.symbol, "long", pos.size)
                        .await
                    {
                        Ok(_) => {
                            info!("✅ 平多仓成功");
                            sleep(Duration::from_secs(1)).await;

                            // 开空仓
                            match exchange
                                .open_short(
                                    &config.symbol,
                                    position_size,
                                    config.leverage,
                                    "cross",
                                    false,
                                )
                                .await
                            {
                                Ok(_) => info!("✅ 开空仓成功"),
                                Err(e) => error!("❌ 开空仓失败: {}", e),
                            }
                        }
                        Err(e) => error!("❌ 平多仓失败: {}", e),
                    }
                }
                _ => {}
            }
        }

        "HOLD" => {
            info!("⏸️  观望中，不执行交易");
            if let Some(pos) = current_position {
                info!("   当前持仓: {} {:.6} BTC", pos.side, pos.size);
                info!("   盈亏: ${:.2}", pos.unrealized_pnl);
            }
        }

        _ => {
            info!("⏭️  未知信号类型: {}", signal.signal);
        }
    }

    Ok(())
}

// 简单的随机数生成（用于模拟 K 线）
mod rand {
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn random() -> f64 {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        (nanos % 10000) as f64 / 10000.0
    }
}
