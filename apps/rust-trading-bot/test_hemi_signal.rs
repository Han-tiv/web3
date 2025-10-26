// 测试HEMIUSDT信号解析
fn main() {
    let signal_text = "HEMIUSDT - 看跌📉 - 🥷猎龙忍者🥷
-----------------------------------
时间: 2025-09-30 19:26:16
当前价格: 0.09196 USDT
贪婪恐慌等级: 正常水平
🌈OI/MC：0.3（25.7M/91.8M）
技术分析雷达: 未触发
资金雷达(现货): 无数据/成交偏低
资金雷达(期货): 无数据/成交偏低
✅猎龙忍者: 多头出逃（狩猎指数-202）
触发雷达数: 1/3";

    // 测试平仓信号正则 (优先级1)
    if let Some(regex) = regex::Regex::new(r"(\w+USDT)\s*-\s*看(?:跌|涨)跟踪结束").ok() {
        if let Some(caps) = regex.captures(signal_text) {
            let symbol = caps.get(1).unwrap().as_str();
            println!("✅ 匹配到平仓信号: Close({})", symbol);
            return;
        }
    }

    // 测试开仓信号正则 (优先级2)
    if let Some(regex) = regex::Regex::new(r"(\w+USDT)\s*-\s*看(跌|涨)(?!.*跟踪)").ok() {
        if let Some(caps) = regex.captures(signal_text) {
            let symbol = caps.get(1).unwrap().as_str();
            let direction = caps.get(2).unwrap().as_str();
            match direction {
                "涨" => println!("✅ 匹配到开多信号: OpenLong({})", symbol),
                "跌" => println!("✅ 匹配到开空信号: OpenShort({})", symbol),
                _ => println!("❌ 未知方向: {}", direction),
            }
            return;
        }
    }

    println!("❌ 未匹配到任何信号");
}