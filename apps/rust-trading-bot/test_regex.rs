use regex::Regex;

fn main() {
    let text = "TUTUSDT - 看跌📉 - 🥷猎龙忍者🥷\n-----------------------------------\n时间: 2025-10-02 20:10:20\n当前价格: 0.08309 USDT\n贪婪恐慌等级: 正常水平\n🌈OI/MC：0.7（49.6M/70.4M）\n技术分析雷达: 未触发\n资金雷达(现货): 未触发 (分数: 45.6)\n资金雷达(期货): 未触发 (分数: 38.4)\n✅猎龙忍者: 多头出逃（狩猎指数-245）\n触发雷达数: 1/3";
    
    // 测试当前的正则表达式
    let open_re = Regex::new(r"(\w+USDT)\s*-\s*看(跌|涨)(?!.*跟踪)").unwrap();
    
    println!("输入文本: {}", text);
    println!();
    
    if let Some(caps) = open_re.captures(text) {
        let symbol = caps.get(1).unwrap().as_str();
        let direction = caps.get(2).unwrap().as_str();
        println!("✅ 匹配成功!");
        println!("币种: {}", symbol);
        println!("方向: {}", direction);
    } else {
        println!("❌ 正则匹配失败");
        
        // 测试简化版本
        let simple_re = Regex::new(r"(\w+USDT)\s*-\s*看(跌|涨)").unwrap();
        if let Some(caps) = simple_re.captures(text) {
            println!("✅ 简化版正则匹配成功!");
            println!("币种: {}", caps.get(1).unwrap().as_str());
            println!("方向: {}", caps.get(2).unwrap().as_str());
        } else {
            println!("❌ 连简化版都匹配失败");
        }
    }
}
