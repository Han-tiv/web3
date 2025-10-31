use anyhow::Result;
use dotenv::dotenv;
use grammers_client::{Client, Config};
use grammers_session::Session;
use serde::{Serialize, Deserialize};
use std::env;
use std::fs::File;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize)]
struct ChannelMessage {
    message_id: i32,
    timestamp: String,
    sender_id: i64,
    sender_name: String,
    content: String,
    has_media: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChannelAnalysis {
    channel_id: i64,
    channel_name: String,
    total_messages: usize,
    date_range: String,
    messages: Vec<ChannelMessage>,
    keywords_stats: KeywordStats,
}

#[derive(Debug, Serialize, Deserialize)]
struct KeywordStats {
    fund_inflow: usize,  // 资金流入
    fund_outflow: usize, // 资金流出
    main_force: usize,   // 主力
    institutions: usize, // 机构
    retail: usize,       // 散户
    whale: usize,        // 巨鲸
    coins_mentioned: std::collections::HashMap<String, usize>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    println!("════════════════════════════════════════════════════════════");
    println!("📊 Telegram 频道消息分析工具 - 主力资金监控版");
    println!("════════════════════════════════════════════════════════════\n");

    // 目标配置
    let channel_id = 2254462672_i64; // valuescan 主力资金监控频道
    let max_messages = 1000; // 分析最近1000条消息

    println!("🎯 目标配置:");
    println!("  频道 ID: {}", channel_id);
    println!("  分析数量: {} 条消息\n", max_messages);

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
        anyhow::bail!("未登录，请先运行登录程序");
    }

    println!("✅ 已连接\n");

    println!("🔍 查找频道...");
    let mut channel = None;
    let mut dialogs = client.iter_dialogs();
    while let Some(dialog) = dialogs.next().await? {
        if dialog.chat.id() == channel_id {
            channel = Some(dialog.chat);
            break;
        }
    }

    let channel = channel.ok_or_else(|| anyhow::anyhow!("找不到频道 {}", channel_id))?;
    let channel_name = channel.name().to_string();
    println!("✅ 找到频道: {}\n", channel_name);

    println!("📨 开始获取消息...");
    println!("⏳ 这可能需要一些时间，请稍候...\n");

    let mut messages = Vec::new();
    let mut count = 0;
    let mut first_timestamp = None;
    let mut last_timestamp = None;

    let mut iter = client.iter_messages(channel);
    while let Some(message) = iter.next().await? {
        count += 1;

        if count % 100 == 0 {
            println!("📊 已获取 {} 条消息", count);
        }

        let timestamp = message.date().to_string();
        if first_timestamp.is_none() {
            first_timestamp = Some(timestamp.clone());
        }
        last_timestamp = Some(timestamp.clone());

        let (sender_id, sender_name) = if let Some(sender) = message.sender() {
            let id = sender.id();
            let name = match sender {
                grammers_client::types::Chat::User(user) => {
                    user.first_name().to_string()
                }
                grammers_client::types::Chat::Channel(ch) => ch.title().to_string(),
                grammers_client::types::Chat::Group(g) => g.title().to_string(),
            };
            (id, name)
        } else {
            (0, "Unknown".to_string())
        };

        messages.push(ChannelMessage {
            message_id: message.id(),
            timestamp,
            sender_id,
            sender_name,
            content: message.text().to_string(),
            has_media: message.media().is_some(),
        });

        if count >= max_messages {
            println!("\n✅ 已达到设定数量 {} 条", max_messages);
            break;
        }
    }

    println!("\n✅ 获取完成！");
    println!("   总消息数: {} 条\n", messages.len());

    if messages.is_empty() {
        println!("❌ 未找到任何消息");
        return Ok(());
    }

    // 分析关键词
    println!("🔍 分析关键词统计...\n");
    let keywords_stats = analyze_keywords(&messages);

    let date_range = format!(
        "{} 至 {}",
        last_timestamp.as_ref().unwrap(),
        first_timestamp.as_ref().unwrap()
    );

    let analysis = ChannelAnalysis {
        channel_id,
        channel_name: channel_name.clone(),
        total_messages: messages.len(),
        date_range,
        messages,
        keywords_stats,
    };

    // 保存 JSON
    let json_filename = format!("channel_{}_analysis.json", channel_id);
    let json_file = File::create(&json_filename)?;
    serde_json::to_writer_pretty(json_file, &analysis)?;
    println!("✅ JSON 报告已保存: {}", json_filename);

    // 生成文本报告
    let txt_filename = format!("channel_{}_analysis.txt", channel_id);
    generate_text_report(&analysis, &txt_filename)?;
    println!("✅ 文本报告已保存: {}", txt_filename);

    // 打印摘要
    print_summary(&analysis);

    Ok(())
}

fn analyze_keywords(messages: &[ChannelMessage]) -> KeywordStats {
    let mut stats = KeywordStats {
        fund_inflow: 0,
        fund_outflow: 0,
        main_force: 0,
        institutions: 0,
        retail: 0,
        whale: 0,
        coins_mentioned: std::collections::HashMap::new(),
    };

    let coin_patterns = vec![
        ("BTC", vec!["btc", "bitcoin", "比特币", "大饼"]),
        ("ETH", vec!["eth", "ethereum", "以太坊", "姨太"]),
        ("BNB", vec!["bnb", "币安币"]),
        ("SOL", vec!["sol", "solana"]),
        ("XRP", vec!["xrp", "瑞波"]),
        ("DOGE", vec!["doge", "狗狗币"]),
        ("ADA", vec!["ada", "艾达币"]),
        ("AVAX", vec!["avax", "雪崩"]),
        ("DOT", vec!["dot", "波卡"]),
        ("MATIC", vec!["matic", "马蹄"]),
    ];

    for msg in messages {
        let content_lower = msg.content.to_lowercase();

        // 资金流向
        if content_lower.contains("流入") || content_lower.contains("买入") 
            || content_lower.contains("净流入") || content_lower.contains("inflow") {
            stats.fund_inflow += 1;
        }
        if content_lower.contains("流出") || content_lower.contains("卖出") 
            || content_lower.contains("净流出") || content_lower.contains("outflow") {
            stats.fund_outflow += 1;
        }

        // 参与者类型
        if content_lower.contains("主力") || content_lower.contains("庄家") {
            stats.main_force += 1;
        }
        if content_lower.contains("机构") || content_lower.contains("institution") {
            stats.institutions += 1;
        }
        if content_lower.contains("散户") || content_lower.contains("retail") {
            stats.retail += 1;
        }
        if content_lower.contains("巨鲸") || content_lower.contains("whale") 
            || content_lower.contains("大户") {
            stats.whale += 1;
        }

        // 币种提及
        for (coin, patterns) in &coin_patterns {
            for pattern in patterns {
                if content_lower.contains(pattern) {
                    *stats.coins_mentioned.entry(coin.to_string()).or_insert(0) += 1;
                    break;
                }
            }
        }
    }

    stats
}

fn generate_text_report(analysis: &ChannelAnalysis, filename: &str) -> Result<()> {
    let mut file = File::create(filename)?;

    writeln!(file, "频道消息分析报告")?;
    writeln!(file, "频道名称: {}", analysis.channel_name)?;
    writeln!(file, "频道 ID: {}", analysis.channel_id)?;
    writeln!(file, "总消息数: {}", analysis.total_messages)?;
    writeln!(file, "时间范围: {}", analysis.date_range)?;
    writeln!(file, "============================================================\n")?;

    writeln!(file, "【关键词统计】")?;
    writeln!(file, "资金流入提及: {} 次", analysis.keywords_stats.fund_inflow)?;
    writeln!(file, "资金流出提及: {} 次", analysis.keywords_stats.fund_outflow)?;
    writeln!(file, "主力提及: {} 次", analysis.keywords_stats.main_force)?;
    writeln!(file, "机构提及: {} 次", analysis.keywords_stats.institutions)?;
    writeln!(file, "散户提及: {} 次", analysis.keywords_stats.retail)?;
    writeln!(file, "巨鲸提及: {} 次", analysis.keywords_stats.whale)?;
    writeln!(file)?;

    writeln!(file, "【币种提及排行】")?;
    let mut coins: Vec<_> = analysis.keywords_stats.coins_mentioned.iter().collect();
    coins.sort_by(|a, b| b.1.cmp(a.1));
    for (coin, count) in coins.iter().take(10) {
        writeln!(file, "{}: {} 次", coin, count)?;
    }
    writeln!(file)?;

    writeln!(file, "============================================================")?;
    writeln!(file, "【最近 20 条消息】\n")?;

    for (i, msg) in analysis.messages.iter().take(20).enumerate() {
        writeln!(file, "【消息 {}】", i + 1)?;
        writeln!(file, "时间: {}", msg.timestamp)?;
        writeln!(file, "发送者: {} (ID: {})", msg.sender_name, msg.sender_id)?;
        writeln!(file, "消息 ID: {}", msg.message_id)?;
        if msg.has_media {
            writeln!(file, "包含媒体: 是")?;
        }
        writeln!(file, "\n{}\n", msg.content)?;
        writeln!(file, "------------------------------------------------------------")?;
    }

    Ok(())
}

fn print_summary(analysis: &ChannelAnalysis) {
    println!("\n════════════════════════════════════════════════════════════");
    println!("📊 频道分析摘要");
    println!("════════════════════════════════════════════════════════════\n");

    println!("📢 频道信息:");
    println!("  名称: {}", analysis.channel_name);
    println!("  ID: {}", analysis.channel_id);
    println!("  消息数: {}", analysis.total_messages);
    println!("  时间范围: {}\n", analysis.date_range);

    println!("💰 资金流向:");
    println!("  流入提及: {} 次", analysis.keywords_stats.fund_inflow);
    println!("  流出提及: {} 次", analysis.keywords_stats.fund_outflow);
    let net_sentiment = analysis.keywords_stats.fund_inflow as i32 - analysis.keywords_stats.fund_outflow as i32;
    println!("  净情绪: {} ({})", 
        net_sentiment,
        if net_sentiment > 0 { "偏多" } else if net_sentiment < 0 { "偏空" } else { "中性" }
    );
    println!();

    println!("👥 参与者类型:");
    println!("  主力: {} 次", analysis.keywords_stats.main_force);
    println!("  机构: {} 次", analysis.keywords_stats.institutions);
    println!("  巨鲸: {} 次", analysis.keywords_stats.whale);
    println!("  散户: {} 次", analysis.keywords_stats.retail);
    println!();

    println!("🪙 热门币种 (Top 10):");
    let mut coins: Vec<_> = analysis.keywords_stats.coins_mentioned.iter().collect();
    coins.sort_by(|a, b| b.1.cmp(a.1));
    for (i, (coin, count)) in coins.iter().take(10).enumerate() {
        println!("  {}. {}: {} 次", i + 1, coin, count);
    }

    println!("\n════════════════════════════════════════════════════════════");
}
