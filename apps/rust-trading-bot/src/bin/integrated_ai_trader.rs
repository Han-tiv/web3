/// 集成AI交易系统 - 整合主力资金监控 + DeepSeek AI + 多交易所执行
/// 
/// 功能：
/// 1. 监控Telegram主力资金频道(Valuescan 2254462672)
/// 2. 筛选Alpha/FOMO高潜力币种
/// 3. 获取技术数据（K线、指标、关键位）
/// 4. DeepSeek AI综合分析决策
/// 5. 多交易所并发执行
/// 6. 严格风控管理

use anyhow::Result;
use dotenv::dotenv;
use grammers_client::{Client, Config, Update};
use grammers_session::Session;
use log::{info, warn, error};
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use regex::Regex;

use rust_trading_bot::{
    binance_client::BinanceClient,
    exchange_trait::ExchangeClient,
    deepseek_client::{DeepSeekClient, Kline},
    technical_analysis::TechnicalAnalyzer,
    smart_money_tracker::SmartMoneyTracker,
    key_level_finder::KeyLevelFinder,
};

#[derive(Debug, Clone)]
struct FundAlert {
    coin: String,
    alert_type: AlertType,
    price: f64,
    change_24h: f64,
    fund_type: String,
    timestamp: DateTime<Utc>,
    raw_message: String,
}

#[derive(Debug, Clone, PartialEq)]
enum AlertType {
    AlphaOpportunity,
    FomoSignal,
    FundInflow,
    FundEscape,
}

struct IntegratedAITrader {
    telegram_client: Arc<Client>,
    exchange: Arc<BinanceClient>,
    deepseek: Arc<DeepSeekClient>,
    analyzer: Arc<TechnicalAnalyzer>,
    tracker: Arc<SmartMoneyTracker>,
    level_finder: Arc<KeyLevelFinder>,
    
    // 配置
    fund_channel_id: i64,
    alpha_keywords: Vec<String>,
    fomo_keywords: Vec<String>,
    
    // 交易配置
    base_position_usdt: f64,
    leverage: u32,
    
    // 状态跟踪
    tracked_coins: Arc<RwLock<HashMap<String, FundAlert>>>,
}

impl IntegratedAITrader {
    async fn new(
        telegram_client: Client,
        exchange: BinanceClient,
        deepseek_api_key: String,
    ) -> Self {
        Self {
            telegram_client: Arc::new(telegram_client),
            exchange: Arc::new(exchange),
            deepseek: Arc::new(DeepSeekClient::new(deepseek_api_key)),
            analyzer: Arc::new(TechnicalAnalyzer::new()),
            tracker: Arc::new(SmartMoneyTracker::new()),
            level_finder: Arc::new(KeyLevelFinder::new()),
            
            fund_channel_id: 2254462672_i64, // Valuescan
            alpha_keywords: vec![
                "alpha".to_string(),
                "新币".to_string(),
                "上线".to_string(),
                "首发".to_string(),
                "binance".to_string(),
                "币安".to_string(),
            ],
            fomo_keywords: vec![
                "暴涨".to_string(),
                "拉升".to_string(),
                "突破".to_string(),
                "异动".to_string(),
                "急拉".to_string(),
                "爆发".to_string(),
            ],
            
            base_position_usdt: 6.0,
            leverage: 5,
            
            tracked_coins: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 解析资金异动消息
    fn parse_fund_alert(&self, text: &str) -> Option<FundAlert> {
        // 提取币种 $COIN格式
        let coin_regex = Regex::new(r"\$([A-Z0-9]+)").ok()?;
        let coin = coin_regex.captures(text)?.get(1)?.as_str().to_string();

        // 判断消息类型
        let alert_type = if text.contains("出逃") || text.contains("撤离") {
            AlertType::FundEscape
        } else if text.contains("【资金异动】") {
            AlertType::FundInflow
        } else {
            return None;
        };

        // 提取价格
        let price_regex = Regex::new(r"现价[:：]\s*\$?([\d.]+)").ok()?;
        let price: f64 = price_regex
            .captures(text)?
            .get(1)?
            .as_str()
            .parse()
            .ok()?;

        // 提取24H涨跌幅
        let change_regex = Regex::new(r"24H[:：]\s*([+-]?[\d.]+)%").ok()?;
        let change_24h: f64 = change_regex
            .captures(text)?
            .get(1)?
            .as_str()
            .parse()
            .ok()?;

        // 提取资金类型
        let fund_type = if text.contains("合约") {
            "合约".to_string()
        } else if text.contains("现货") {
            "现货".to_string()
        } else {
            "未知".to_string()
        };

        Some(FundAlert {
            coin,
            alert_type,
            price,
            change_24h,
            fund_type,
            timestamp: Utc::now(),
            raw_message: text.to_string(),
        })
    }

    /// 判断是否为Alpha/FOMO机会
    fn is_alpha_or_fomo(&self, alert: &FundAlert) -> bool {
        let message_lower = alert.raw_message.to_lowercase();
        
        // 检查Alpha关键词
        let is_alpha = self.alpha_keywords.iter()
            .any(|kw| message_lower.contains(kw));
        
        // 检查FOMO关键词或高涨幅
        let is_fomo = self.fomo_keywords.iter()
            .any(|kw| message_lower.contains(kw))
            || alert.change_24h > 10.0;

        is_alpha || is_fomo
    }

    /// 更新分类
    fn classify_alert(&self, alert: &mut FundAlert) {
        let message_lower = alert.raw_message.to_lowercase();
        
        if self.alpha_keywords.iter().any(|kw| message_lower.contains(kw)) {
            alert.alert_type = AlertType::AlphaOpportunity;
        } else if self.fomo_keywords.iter().any(|kw| message_lower.contains(kw)) 
            || alert.change_24h > 10.0 {
            alert.alert_type = AlertType::FomoSignal;
        }
    }

    /// 处理新消息
    async fn handle_message(&self, text: &str) -> Result<()> {
        // 解析资金异动
        if let Some(mut alert) = self.parse_fund_alert(text) {
            // 过滤掉出逃信号（日内交易不关注）
            if alert.alert_type == AlertType::FundEscape {
                info!("⚠️  主力出逃信号: {} - 忽略", alert.coin);
                return Ok(());
            }

            // 检查是否为Alpha/FOMO机会
            if !self.is_alpha_or_fomo(&alert) {
                info!("📊 普通资金流入: {} - 忽略（非Alpha/FOMO）", alert.coin);
                return Ok(());
            }

            // 更新分类
            self.classify_alert(&mut alert);

            info!("\n🔥 发现{}机会: {} 💰", 
                match alert.alert_type {
                    AlertType::AlphaOpportunity => "Alpha",
                    AlertType::FomoSignal => "FOMO",
                    _ => "未知",
                },
                alert.coin
            );
            info!("   价格: ${:.4} | 24H: {:+.2}% | 类型: {}", 
                alert.price, alert.change_24h, alert.fund_type);

            // 保存到跟踪列表
            let mut coins = self.tracked_coins.write().await;
            coins.insert(alert.coin.clone(), alert.clone());
            drop(coins);

            // 触发AI分析
            self.analyze_and_trade(alert).await?;
        }

        Ok(())
    }

    /// AI分析并执行交易
    async fn analyze_and_trade(&self, alert: FundAlert) -> Result<()> {
        info!("🧠 开始AI分析: {}", alert.coin);

        // 1. 获取K线数据
        let symbol = format!("{}/USDT", alert.coin);
        let klines = match self.exchange.get_klines(&symbol, "15m", Some(100)).await {
            Ok(data) => {
                data.iter().map(|candle| Kline {
                    timestamp: candle[0] as i64,
                    open: candle[1],
                    high: candle[2],
                    low: candle[3],
                    close: candle[4],
                    volume: candle[5],
                }).collect::<Vec<_>>()
            }
            Err(e) => {
                warn!("❌ 获取{}K线失败: {}", symbol, e);
                return Ok(());
            }
        };

        if klines.len() < 20 {
            warn!("⚠️  K线数据不足: {} (需要至少20根)", klines.len());
            return Ok(());
        }

        // 2. 计算技术指标
        let indicators = self.analyzer.calculate_indicators(&klines);
        
        // 3. 识别关键位
        let key_levels = self.level_finder.identify_key_levels(&klines, 24);
        
        // 4. 构建增强的DeepSeek Prompt
        let current_price = klines.last().unwrap().close;
        let prompt = self.build_enhanced_prompt(&alert, &klines, &indicators, &key_levels, current_price);
        
        info!("📝 发送给DeepSeek AI分析...");
        
        // 5. 调用DeepSeek API（这里是模拟，实际需要实现HTTP调用）
        info!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("📊 DeepSeek AI Prompt:");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("{}", prompt);
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        
        // TODO: 实际调用DeepSeek API
        // let decision = self.deepseek.analyze(&prompt).await?;
        
        // 6. 执行交易（演示模式）
        info!("💡 当前为演示模式，不执行实际交易");
        info!("💡 要启用交易，请实现DeepSeek API调用和交易执行逻辑\n");
        
        Ok(())
    }

    /// 构建增强的DeepSeek Prompt
    fn build_enhanced_prompt(
        &self,
        alert: &FundAlert,
        klines: &[Kline],
        indicators: &rust_trading_bot::technical_analysis::TechnicalIndicators,
        key_levels: &[rust_trading_bot::key_level_finder::KeyLevel],
        current_price: f64,
    ) -> String {
        let alert_type_desc = match alert.alert_type {
            AlertType::AlphaOpportunity => "🎯 Alpha机会（新币/首发/高潜力）",
            AlertType::FomoSignal => "🔥 FOMO信号（快速拉升/突破/高涨幅）",
            _ => "资金流入",
        };

        // 找到最近的关键位
        let (nearest_support, nearest_resistance) = 
            self.level_finder.find_nearest_levels(key_levels, current_price);

        format!(
            r#"你是专业的日内交易分析师，现在有一个{alert_type}的交易机会需要评估。

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 币种: ${}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

💰 【主力资金信号】
- 信号类型: {}
- 当前价格: ${:.6}
- 24H涨幅: {:+.2}%
- 资金类型: {}
- 发现时间: {}

📈 【技术指标 (15分钟)】
- RSI(14): {:.2}
- MACD: {:.4} (信号线: {:.4}, 柱状: {:.4})
- 布林带: 上轨${:.4} | 中轨${:.4} | 下轨${:.4}
- SMA5: ${:.4} | SMA20: ${:.4} | SMA50: ${:.4}
- 当前价格位置: {}

🎯 【主力关键位】
{}

📊 【市场状态】
- 当前价格: ${:.4}
- 24H最高: ${:.4}
- 24H最低: ${:.4}
- 成交量(最近): {:.2}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🎯 【日内交易决策要求】
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

【交易特点】
- 目标: 日内波段交易
- 持仓时间: 30分钟 - 4小时
- 预期收益: 3-10%
- 最大风险: 2%

【Alpha/FOMO币种特殊考虑】
1. **高波动性** - 快速拉升也可能快速回落
2. **流动性风险** - 新币可能流动性不足
3. **消息驱动** - 热点消息退潮风险
4. **获利回吐** - FOMO后的快速抛售

【整合主力关键位策略】
✅ **入场条件**:
- 价格在主力关键位附近(±2%) + 未破位
- {}
- 资金流入持续、未见主力出逃信号
- 设置好止损位（关键位-2%）

❌ **不入场条件**:
- 已大幅拉升（>30%）且无回调
- RSI>70严重超买
- 破主力关键位
- 流动性极差

🎯 **止盈止损**:
- 止盈1: +3% 减半仓
- 止盈2: +5% 清仓
- 止损: 主力关键位-2%或入场价-2%（取近的）
- 时间止损: 4小时未突破止盈位则离场

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📝 【输出要求】
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

请以JSON格式输出交易决策：
{{
    "signal": "BUY|HOLD|SKIP",
    "confidence": "HIGH|MEDIUM|LOW",
    "entry_price": 建议入场价格,
    "stop_loss": 止损价格,
    "take_profit_1": 第一目标（减半仓）,
    "take_profit_2": 第二目标（清仓）,
    "position_size": "SMALL|MEDIUM|LARGE",
    "reason": "详细理由（100字以内）",
    "risks": ["风险点1", "风险点2", "风险点3"],
    "time_horizon": "预计持仓时间（分钟）",
    "priority": "HIGH|MEDIUM|LOW"
}}

【特别说明】
- BUY: 强烈推荐入场
- HOLD: 等待更好时机
- SKIP: 不建议交易
- position_size: SMALL(1%), MEDIUM(2%), LARGE(3%) 占总仓位比例

请综合分析后给出明确决策！
"#,
            alert_type = alert_type_desc,
            alert.coin,
            alert_type_desc,
            alert.price,
            alert.change_24h,
            alert.fund_type,
            alert.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            indicators.rsi,
            indicators.macd,
            indicators.macd_signal,
            indicators.macd - indicators.macd_signal,
            indicators.bb_upper,
            indicators.bb_middle,
            indicators.bb_lower,
            indicators.sma_5,
            indicators.sma_20,
            indicators.sma_50,
            self.get_bb_position(current_price, indicators),
            self.format_key_levels(key_levels, current_price, &nearest_support, &nearest_resistance),
            current_price,
            klines.iter().map(|k| k.high).fold(f64::MIN, f64::max),
            klines.iter().map(|k| k.low).fold(f64::MAX, f64::min),
            klines.last().unwrap().volume,
            self.format_entry_condition(&nearest_support, &nearest_resistance, current_price),
        )
    }

    fn get_bb_position(&self, price: f64, indicators: &rust_trading_bot::technical_analysis::TechnicalIndicators) -> &str {
        let upper_dist = (indicators.bb_upper - price).abs();
        let middle_dist = (indicators.bb_middle - price).abs();
        let lower_dist = (indicators.bb_lower - price).abs();

        let min_dist = upper_dist.min(middle_dist).min(lower_dist);

        if min_dist == upper_dist {
            "上轨区（超买风险）"
        } else if min_dist == lower_dist {
            "下轨区（超卖机会）"
        } else {
            "中轨区（正常范围）"
        }
    }

    fn format_key_levels(
        &self,
        levels: &[rust_trading_bot::key_level_finder::KeyLevel],
        current_price: f64,
        nearest_support: &Option<rust_trading_bot::key_level_finder::KeyLevel>,
        nearest_resistance: &Option<rust_trading_bot::key_level_finder::KeyLevel>,
    ) -> String {
        let mut result = String::new();
        
        if let Some(support) = nearest_support {
            let dist_pct = ((current_price - support.price) / current_price) * 100.0;
            result.push_str(&format!(
                "- 最近支撑位: ${:.4} (距离-{:.2}%, 强度{:.0}分)\n",
                support.price, dist_pct, support.strength
            ));
        }
        
        if let Some(resistance) = nearest_resistance {
            let dist_pct = ((resistance.price - current_price) / current_price) * 100.0;
            result.push_str(&format!(
                "- 最近阻力位: ${:.4} (距离+{:.2}%, 强度{:.0}分)\n",
                resistance.price, dist_pct, resistance.strength
            ));
        }
        
        if result.is_empty() {
            result = "- 未识别到明显关键位\n".to_string();
        }
        
        result
    }

    fn format_entry_condition(
        &self,
        nearest_support: &Option<rust_trading_bot::key_level_finder::KeyLevel>,
        nearest_resistance: &Option<rust_trading_bot::key_level_finder::KeyLevel>,
        current_price: f64,
    ) -> String {
        match (nearest_support, nearest_resistance) {
            (Some(support), Some(resistance)) => {
                let support_dist = ((current_price - support.price) / current_price) * 100.0;
                let resistance_dist = ((resistance.price - current_price) / current_price) * 100.0;
                
                if support_dist < 2.0 {
                    format!("在支撑位附近(距离{:.2}%)，回踩机会", support_dist)
                } else if resistance_dist < 2.0 {
                    format!("接近阻力位(距离{:.2}%)，突破确认后入场", resistance_dist)
                } else {
                    "在支撑与阻力之间，等待明确方向".to_string()
                }
            }
            _ => "关键位不明确，谨慎操作".to_string(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("🚀 集成AI交易系统 - Alpha/FOMO日内交易版");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // 读取配置
    let telegram_api_id = env::var("TELEGRAM_API_ID")?.parse::<i32>()?;
    let telegram_api_hash = env::var("TELEGRAM_API_HASH")?;
    let deepseek_api_key = env::var("DEEPSEEK_API_KEY")?;
    let binance_api_key = env::var("BINANCE_API_KEY")?;
    let binance_secret = env::var("BINANCE_SECRET")?;
    let testnet = env::var("BINANCE_TESTNET")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false);

    info!("🎯 系统配置:");
    info!("  监控频道: Valuescan (2254462672)");
    info!("  监控类型: Alpha机会 + FOMO信号");
    info!("  交易策略: 主力关键位 + 日内波段");
    info!("  AI引擎: DeepSeek");
    info!("  交易所: Binance");
    info!("  测试模式: {}\n", if testnet { "是" } else { "否" });

    // 连接Telegram
    info!("🔄 连接到 Telegram...");
    let telegram_client = Client::connect(Config {
        session: Session::load_file_or_create("session.session")?,
        api_id: telegram_api_id,
        api_hash: telegram_api_hash.clone(),
        params: Default::default(),
    })
    .await?;

    if !telegram_client.is_authorized().await? {
        anyhow::bail!("❌ 未登录，请先运行: cargo run --bin get_channels");
    }

    info!("✅ Telegram已连接\n");

    // 初始化交易所
    let exchange = BinanceClient::new(binance_api_key, binance_secret, testnet);
    info!("✅ Binance客户端已初始化\n");

    // 创建集成交易器
    let trader = Arc::new(
        IntegratedAITrader::new(telegram_client, exchange, deepseek_api_key).await
    );

    info!("📡 开始实时监控...");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // 实时监控循环
    loop {
        match trader.telegram_client.next_update().await {
            Ok(Update::NewMessage(message)) if !message.outgoing() => {
                if let Some(chat) = message.chat() {
                    if chat.id() == trader.fund_channel_id {
                        let text = message.text();
                        if !text.is_empty() {
                            if let Err(e) = trader.handle_message(text).await {
                                error!("❌ 处理消息错误: {}", e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!("❌ Telegram连接错误: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
            _ => {}
        }
    }
}
