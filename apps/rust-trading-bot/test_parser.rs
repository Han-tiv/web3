use regex::Regex;

#[derive(Debug)]
struct TestAlert {
    coin: String,
    alert_type: String,
    price: f64,
    change_24h: f64,
    fund_type: String,
}

fn parse_fund_alert(text: &str) -> Option<TestAlert> {
    // 提取币种 $COIN格式
    let coin_regex = Regex::new(r"\$([A-Z0-9]+)").ok()?;
    let coin = coin_regex.captures(text)?.get(1)?.as_str().to_string();

    // 判断消息类型
    let alert_type = if text.contains("出逃") || text.contains("撤离") {
        "FundEscape"
    } else if text.contains("【资金异动】") {
        "FundInflow"
    } else {
        return None;
    };

    // 提取价格
    let price_regex = Regex::new(r"现价[:：]\s*\$?([\d.]+)").ok()?;
    let price: f64 = price_regex.captures(text)?.get(1)?.as_str().parse().ok()?;

    // 提取24H涨跌幅
    let change_regex = Regex::new(r"24H[:：]\s*([+-]?[\d.]+)%").ok()?;
    let change_24h: f64 = change_regex.captures(text)?.get(1)?.as_str().parse().ok()?;

    // 提取资金类型
    let fund_type = if text.contains("合约") {
        "合约"
    } else if text.contains("现货") {
        "现货"
    } else {
        "未知"
    };

    Some(TestAlert {
        coin,
        alert_type: alert_type.to_string(),
        price,
        change_24h,
        fund_type: fund_type.to_string(),
    })
}

fn main() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Valuescan 消息解析测试");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // 测试用例 1: 标准资金流入信号
    let test_cases = vec![
        // 案例1: 标准格式
        r#"【资金异动】$BTC
现价: $45000.50
24H: +5.2%
类型: 合约
主力资金流入10亿"#,
        // 案例2: 中文冒号
        r#"【资金异动】$ETH
现价：3500.25
24H：-3.5%
类型：现货"#,
        // 案例3: 出逃信号
        r#"⚠️ 主力出逃 $SOL
现价: $120.30
24H: -8.5%
合约资金撤离"#,
        // 案例4: 无美元符号价格
        r#"【资金异动】$DOGE
现价: 0.08
24H: +12.3%
合约"#,
        // 案例5: 应该失败 - 缺少【资金异动】
        r#"$BNB 价格上涨
现价: 350
24H: +2%"#,
        // 案例6: 应该失败 - 缺少币种
        r#"【资金异动】
现价: 100
24H: +5%"#,
        // 案例7: 真实 Valuescan 格式
        r#"【资金异动】主力资金流入 $BTC 合约
💰 现价: $67890.50
📊 24H: +8.2%
🔥 合约资金净流入: 15.2亿"#,
    ];

    for (i, test_msg) in test_cases.iter().enumerate() {
        println!("测试用例 {}:", i + 1);
        println!("输入消息:");
        println!("{}", test_msg);
        println!();

        match parse_fund_alert(test_msg) {
            Some(alert) => {
                println!("✅ 解析成功:");
                println!("   币种: {}", alert.coin);
                println!("   类型: {}", alert.alert_type);
                println!("   价格: ${}", alert.price);
                println!("   24H涨跌: {:+.2}%", alert.change_24h);
                println!("   资金类型: {}", alert.fund_type);
            }
            None => {
                println!("❌ 解析失败 - 不符合预期格式");
            }
        }
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    }
}
