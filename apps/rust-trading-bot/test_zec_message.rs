use regex::Regex;

fn parse_fund_alert(text: &str) -> Option<(String, f64, f64, String)> {
    println!("原始消息:");
    println!("{}", text);
    println!("\n开始解析...\n");

    // 1. 检查【资金异动】关键字
    let has_alert_keyword = text.contains("【资金异动】");
    let has_escape = text.contains("出逃") || text.contains("撤离");
    println!("检查关键字:");
    println!("  - 【资金异动】: {}", has_alert_keyword);
    println!("  - 出逃/撤离: {}", has_escape);

    if !has_alert_keyword && !has_escape {
        // 增强关键字校验：资金异动缺失时允许 Alpha/FOMO 替代
        let has_alpha = text.contains("【Alpha】");
        let has_fomo = text.contains("【FOMO】");
        println!("  - 【Alpha】: {}", has_alpha);
        println!("  - 【FOMO】: {}", has_fomo);

        if !has_alpha && !has_fomo {
            println!("  ❌ 缺少必需关键字 (【资金异动】/【Alpha】/【FOMO】 或 出逃/撤离)\n");
            return None;
        }
    } else {
        println!("  ✅ 找到关键字");
    }

    // 2. 提取币种
    let coin_regex = Regex::new(r"\$([A-Z0-9]+)").ok()?;
    if let Some(cap) = coin_regex.captures(text) {
        let coin = cap.get(1)?.as_str();
        println!("  ✅ 币种: ${}", coin);
    } else {
        println!("  ❌ 未找到币种 ($COIN格式)\n");
        return None;
    }
    let coin = coin_regex.captures(text)?.get(1)?.as_str().to_string();

    // 3. 提取价格
    let price_regex = Regex::new(r"现价[:：]\s*\$?([\d.]+)").ok()?;
    if let Some(cap) = price_regex.captures(text) {
        let price = cap.get(1)?.as_str();
        println!("  ✅ 现价: ${}", price);
    } else {
        println!("  ❌ 未找到现价 (格式: 现价: $X 或 现价：X)\n");
        return None;
    }
    let price: f64 = price_regex.captures(text)?.get(1)?.as_str().parse().ok()?;

    // 4. 提取24H涨跌幅
    let change_regex = Regex::new(r"24H[:：]\s*([+-]?[\d.]+)%").ok()?;
    if let Some(cap) = change_regex.captures(text) {
        let change = cap.get(1)?.as_str();
        println!("  ✅ 24H涨跌: {}%", change);
    } else {
        println!("  ❌ 未找到24H涨跌 (格式: 24H: ±X% 或 24H：±X%)\n");
        return None;
    }
    let change_24h: f64 = change_regex.captures(text)?.get(1)?.as_str().parse().ok()?;

    // 5. 提取资金类型
    let fund_type = if text.contains("合约") {
        "合约"
    } else if text.contains("现货") {
        "现货"
    } else {
        "未知"
    };
    println!("  ✅ 类型: {}", fund_type);

    println!("\n✅ 解析成功!");
    Some((coin, price, change_24h, fund_type.to_string()))
}

fn main() {
    let zec_message = r#"⭐ 【Alpha】$ZEC
━━━━━━━━━
💰 资金状态: 持续流入
💵 现价: $438.96
📉 24H: -6.52%
📊 类型: 合约

💡 潜力标的，可关注后续表现
━━━━━━━━━
🕐 17:45:27 (UTC+8)"#;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("测试 ZEC 消息解析");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    match parse_fund_alert(zec_message) {
        Some((coin, price, change, fund_type)) => {
            println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("最终结果:");
            println!("  币种: {}", coin);
            println!("  价格: ${}", price);
            println!("  24H: {:+.2}%", change);
            println!("  类型: {}", fund_type);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        }
        None => {
            println!("\n❌ 解析失败 - 消息格式不符合预期");
        }
    }
}
