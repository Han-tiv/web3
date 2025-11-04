/// 主力资金监控机器人 - 专注Alpha/FOMO币种的日内交易
///
/// 功能：
/// 1. 实时监控Valuescan频道(2254462672)
/// 2. 解析资金异动消息，提取币种信息
/// 3. 筛选alpha/FOMO高潜力币种
/// 4. 获取技术数据（K线、指标）
/// 5. 将数据发送给DeepSeek AI进行决策
use anyhow::Result;
use chrono::{DateTime, Utc};
use dotenv::dotenv;
use grammers_client::{types::Message, Client, Config, Update};
use grammers_session::Session;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FundAlert {
    coin: String,
    alert_type: AlertType,
    price: f64,
    change_24h: f64,
    fund_type: String, // "合约" or "现货"
    timestamp: DateTime<Utc>,
    raw_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum AlertType {
    FundInflow,       // 资金流入
    FundEscape,       // 主力出逃
    AlphaOpportunity, // Alpha机会
    FomoSignal,       // FOMO信号
}

#[derive(Debug, Clone, Serialize)]
struct CoinAnalysis {
    coin: String,
    alert: FundAlert,
    technical_data: Option<TechnicalData>,
    recommendation: String,
}

#[derive(Debug, Clone, Serialize)]
struct TechnicalData {
    current_price: f64,
    volume_24h: f64,
    high_24h: f64,
    low_24h: f64,
    change_1h: f64,
    change_24h: f64,
    rsi_15m: Option<f64>,
    macd_15m: Option<String>,
    bb_position: Option<String>, // "上轨"/"中轨"/"下轨"
    funding_rate: Option<f64>,   // 资金费率
}

struct FundMonitor {
    client: Arc<Client>,
    channel_id: i64,
    tracked_coins: Arc<RwLock<HashMap<String, FundAlert>>>,
    alpha_keywords: Vec<String>,
    fomo_keywords: Vec<String>,
}

impl FundMonitor {
    async fn new(client: Client, channel_id: i64) -> Self {
        Self {
            client: Arc::new(client),
            channel_id,
            tracked_coins: Arc::new(RwLock::new(HashMap::new())),
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
        }
    }

    /// 解析资金异动消息
    fn parse_fund_alert(&self, message: &str) -> Option<FundAlert> {
        // 提取币种 $COIN格式
        let coin_regex = Regex::new(r"\$([A-Z0-9]+)").ok()?;
        let coin = coin_regex.captures(message)?.get(1)?.as_str().to_string();

        // 判断消息类型
        let alert_type = if message.contains("出逃") || message.contains("撤离") {
            AlertType::FundEscape
        } else if message.contains("【资金异动】") {
            AlertType::FundInflow
        } else {
            return None;
        };

        // 提取价格
        let price_regex = Regex::new(r"现价[:：]\s*\$?([\d.]+)").ok()?;
        let price: f64 = price_regex
            .captures(message)?
            .get(1)?
            .as_str()
            .parse()
            .ok()?;

        // 提取24H涨跌幅
        let change_regex = Regex::new(r"24H[:：]\s*([+-]?[\d.]+)%").ok()?;
        let change_24h: f64 = change_regex
            .captures(message)?
            .get(1)?
            .as_str()
            .parse()
            .ok()?;

        // 提取资金类型
        let fund_type = if message.contains("合约") {
            "合约".to_string()
        } else if message.contains("现货") {
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
            raw_message: message.to_string(),
        })
    }

    /// 判断是否为Alpha/FOMO机会
    fn is_alpha_or_fomo(&self, alert: &FundAlert) -> bool {
        let message_lower = alert.raw_message.to_lowercase();

        // 检查Alpha关键词
        let is_alpha = self
            .alpha_keywords
            .iter()
            .any(|kw| message_lower.contains(kw));

        // 检查FOMO关键词或高涨幅
        let is_fomo = self
            .fomo_keywords
            .iter()
            .any(|kw| message_lower.contains(kw))
            || alert.change_24h > 10.0; // 24H涨幅>10%

        is_alpha || is_fomo
    }

    /// 更新分类
    fn update_alert_type(&self, alert: &mut FundAlert) {
        let message_lower = alert.raw_message.to_lowercase();

        if self
            .alpha_keywords
            .iter()
            .any(|kw| message_lower.contains(kw))
        {
            alert.alert_type = AlertType::AlphaOpportunity;
        } else if self
            .fomo_keywords
            .iter()
            .any(|kw| message_lower.contains(kw))
            || alert.change_24h > 10.0
        {
            alert.alert_type = AlertType::FomoSignal;
        }
    }

    /// 处理新消息
    async fn handle_message(&self, message: Message) -> Result<()> {
        let text = message.text();
        if text.is_empty() {
            return Ok(());
        }

        // 解析资金异动
        if let Some(mut alert) = self.parse_fund_alert(text) {
            // 过滤掉出逃信号（日内交易不关注）
            if alert.alert_type == AlertType::FundEscape {
                println!("⚠️  主力出逃信号: {} - 忽略", alert.coin);
                return Ok(());
            }

            // 检查是否为Alpha/FOMO机会
            if !self.is_alpha_or_fomo(&alert) {
                println!("📊 普通资金流入: {} - 忽略（非Alpha/FOMO）", alert.coin);
                return Ok(());
            }

            // 更新分类
            self.update_alert_type(&mut alert);

            println!(
                "\n🔥 发现{}机会: {} 💰",
                match alert.alert_type {
                    AlertType::AlphaOpportunity => "Alpha",
                    AlertType::FomoSignal => "FOMO",
                    _ => "未知",
                },
                alert.coin
            );
            println!(
                "   价格: ${:.4} | 24H: {:+.2}% | 类型: {}",
                alert.price, alert.change_24h, alert.fund_type
            );

            // 保存到跟踪列表
            let mut coins = self.tracked_coins.write().await;
            coins.insert(alert.coin.clone(), alert.clone());

            // 触发分析
            self.analyze_coin(alert).await?;
        }

        Ok(())
    }

    /// 分析币种并获取技术数据
    async fn analyze_coin(&self, alert: FundAlert) -> Result<()> {
        println!("🔍 正在获取 {} 的技术数据...", alert.coin);

        // TODO: 从交易所API获取技术数据
        // 这里需要根据币种查询Binance/OKX/Bybit等交易所
        let technical_data = self.fetch_technical_data(&alert.coin).await?;

        // 构建分析数据
        let analysis = CoinAnalysis {
            coin: alert.coin.clone(),
            alert: alert.clone(),
            technical_data: Some(technical_data),
            recommendation: String::new(),
        };

        // 保存分析结果
        self.save_analysis(&analysis).await?;

        // 发送给DeepSeek AI
        self.send_to_deepseek(analysis).await?;

        Ok(())
    }

    /// 从交易所获取技术数据（模拟实现）
    async fn fetch_technical_data(&self, coin: &str) -> Result<TechnicalData> {
        // 这里应该调用实际的交易所API
        // 暂时返回模拟数据
        println!("⏳ 从交易所获取 {} 数据...", coin);

        // TODO: 实际实现需要：
        // 1. 查询币种在哪些交易所上市
        // 2. 获取15m K线数据
        // 3. 计算技术指标（RSI、MACD、布林带）
        // 4. 获取资金费率（合约）

        Ok(TechnicalData {
            current_price: 0.0,
            volume_24h: 0.0,
            high_24h: 0.0,
            low_24h: 0.0,
            change_1h: 0.0,
            change_24h: 0.0,
            rsi_15m: None,
            macd_15m: None,
            bb_position: None,
            funding_rate: None,
        })
    }

    /// 保存分析结果到文件
    async fn save_analysis(&self, analysis: &CoinAnalysis) -> Result<()> {
        let filename = format!(
            "analysis_{}_{}.json",
            analysis.coin,
            Utc::now().format("%Y%m%d_%H%M%S")
        );

        let json = serde_json::to_string_pretty(analysis)?;
        tokio::fs::write(&filename, json).await?;

        println!("💾 分析结果已保存: {}", filename);
        Ok(())
    }

    /// 发送给DeepSeek AI进行决策
    async fn send_to_deepseek(&self, analysis: CoinAnalysis) -> Result<()> {
        println!("🧠 发送给 DeepSeek AI 分析...");

        let prompt = self.build_deepseek_prompt(&analysis);

        println!("\n📝 DeepSeek Prompt:");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("{}", prompt);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        // TODO: 实际调用DeepSeek API
        // let response = deepseek_client.analyze(&prompt).await?;

        Ok(())
    }

    /// 构建DeepSeek分析提示词
    fn build_deepseek_prompt(&self, analysis: &CoinAnalysis) -> String {
        let alert_type_desc = match analysis.alert.alert_type {
            AlertType::AlphaOpportunity => "🎯 Alpha机会（新币/首发/高潜力）",
            AlertType::FomoSignal => "🔥 FOMO信号（快速拉升/突破/高涨幅）",
            _ => "资金流入",
        };

        format!(
            r#"你是专业的日内交易分析师，现在有一个{alert_type}的交易机会需要评估。

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 币种: ${coin}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

💰 【资金流向信息】
- 资金类型: {fund_type}
- 当前价格: ${price:.6}
- 24H涨幅: {change_24h:+.2}%
- 发现时间: {timestamp}
- 信号类型: {alert_type}

{technical_section}

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

【决策逻辑】
✅ **入场条件**:
- Alpha币: 确认上市交易所、有交易量、价格稳定
- FOMO币: 突破关键位、放量上涨、趋势延续
- 资金流入持续、未见主力出逃信号
- 设置好止损位（入场价-2%）

❌ **不入场条件**:
- 已大幅拉升（>30%）且无回调
- 流动性极差（24H成交量<100万U）
- 缺乏技术支撑
- 消息面存疑

🎯 **止盈止损**:
- 止盈: +3% 减半仓，+5% 清仓
- 止损: -2% 立即止损
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
- priority: 与其他机会对比的优先级

请综合分析后给出明确决策！
"#,
            alert_type = alert_type_desc,
            coin = analysis.coin,
            fund_type = analysis.alert.fund_type,
            price = analysis.alert.price,
            change_24h = analysis.alert.change_24h,
            timestamp = analysis.alert.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            technical_section = if let Some(ref tech) = analysis.technical_data {
                format!(
                    r#"📈 【技术数据】
- 当前价: ${:.6}
- 24H成交量: ${:.2}
- 24H高点: ${:.6}
- 24H低点: ${:.6}
- 1H涨幅: {:+.2}%
- RSI(15m): {}
- MACD(15m): {}
- 布林带位置: {}
- 资金费率: {}"#,
                    tech.current_price,
                    tech.volume_24h,
                    tech.high_24h,
                    tech.low_24h,
                    tech.change_1h,
                    tech.rsi_15m
                        .map(|r| format!("{:.2}", r))
                        .unwrap_or("N/A".to_string()),
                    tech.macd_15m.as_ref().map(|s| s.as_str()).unwrap_or("N/A"),
                    tech.bb_position
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or("N/A"),
                    tech.funding_rate
                        .map(|r| format!("{:.4}%", r * 100.0))
                        .unwrap_or("N/A".to_string()),
                )
            } else {
                "⚠️  【技术数据获取中...】\n- 正在从交易所获取K线和指标数据".to_string()
            }
        )
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🚀 主力资金监控机器人 - Alpha/FOMO日内交易版");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let channel_id = 2254462672_i64; // Valuescan 主力资金频道

    println!("🎯 监控配置:");
    println!("  频道 ID: {}", channel_id);
    println!("  监控类型: Alpha机会 + FOMO信号");
    println!("  交易策略: 日内波段");
    println!("  持仓周期: 30分钟 - 4小时");
    println!("  目标收益: 3-10%");
    println!("  最大风险: 2%\n");

    let api_id = env::var("TELEGRAM_API_ID")?.parse::<i32>()?;
    let api_hash = env::var("TELEGRAM_API_HASH")?;

    println!("🔄 连接到 Telegram...");

    let client = Client::connect(Config {
        session: Session::load_file_or_create("session.session")?,
        api_id,
        api_hash: api_hash.clone(),
        params: Default::default(),
    })
    .await?;

    if !client.is_authorized().await? {
        anyhow::bail!("❌ 未登录，请先运行登录程序");
    }

    println!("✅ 已连接\n");

    let monitor = FundMonitor::new(client.clone(), channel_id).await;

    println!("📡 开始实时监控...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // 实时监控消息
    loop {
        match client.next_update().await {
            Ok(Update::NewMessage(message)) if !message.outgoing() => {
                if let Some(chat) = message.chat() {
                    if chat.id() == channel_id {
                        if let Err(e) = monitor.handle_message(message).await {
                            eprintln!("❌ 处理消息错误: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ 获取更新错误: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
            _ => {}
        }
    }
}
