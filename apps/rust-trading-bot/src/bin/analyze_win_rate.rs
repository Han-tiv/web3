use anyhow::{bail, Context, Result};
use chrono::NaiveDateTime;
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
struct CloseRecord {
    symbol: String,
    timestamp: Option<NaiveDateTime>,
    raw_time: Option<String>,
    profit_percent: Option<f64>,
    message: Option<String>,
}

fn main() -> Result<()> {
    // 日志路径默认使用当前目录下的 signal_trader.log，可通过命令行参数覆盖
    let log_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "signal_trader.log".to_string());

    let content =
        fs::read_to_string(&log_path).with_context(|| format!("读取日志文件失败: {}", log_path))?;

    let records = parse_close_records(&content)?;

    if records.is_empty() {
        println!("未在日志中找到任何平仓记录，无法计算胜率。");
        return Ok(());
    }

    report_statistics(&log_path, &records);

    Ok(())
}

fn parse_close_records(content: &str) -> Result<Vec<CloseRecord>> {
    let mut records = Vec::new();
    let mut last_timestamp: Option<NaiveDateTime> = None;
    let mut last_timestamp_raw: Option<String> = None;
    let mut last_profit: Option<f64> = None;
    let mut last_message: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(rest) = trimmed.strip_prefix("时间:") {
            let time_str = rest.trim();
            last_timestamp_raw = Some(time_str.to_string());
            last_timestamp = NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S").ok();
        }

        if let Some(rest) = trimmed.strip_prefix("盈利空间:") {
            let value_str = rest.trim().trim_end_matches('%').trim();
            last_profit = value_str.parse::<f64>().ok();
        }

        if let Some(rest) = trimmed.strip_prefix("完整内容:") {
            last_message = Some(rest.trim().to_string());
        }

        if let Some(rest) = trimmed.strip_prefix("🎯 检测到信号: Close(\"") {
            if let Some(end) = rest.find("\")") {
                let symbol = rest[..end].to_string();
                let record = CloseRecord {
                    symbol,
                    timestamp: last_timestamp,
                    raw_time: last_timestamp_raw.clone(),
                    profit_percent: last_profit,
                    message: last_message.clone(),
                };
                records.push(record);
                last_profit = None;
            } else {
                bail!("无法解析 Close 信号行: {}", trimmed);
            }
        }
    }

    Ok(records)
}

fn report_statistics(log_path: &str, records: &[CloseRecord]) {
    let mut wins = 0usize;
    let mut losses = 0usize;
    let mut neutral = 0usize;
    let mut unknown = 0usize;
    let mut total_profit = 0.0;
    let mut total_known = 0usize;
    let mut earliest: Option<NaiveDateTime> = None;
    let mut latest: Option<NaiveDateTime> = None;

    let mut symbol_counter: HashMap<String, usize> = HashMap::new();

    for record in records {
        *symbol_counter.entry(record.symbol.clone()).or_default() += 1;

        if let Some(ts) = record.timestamp {
            earliest = match earliest {
                Some(existing) if existing <= ts => Some(existing),
                _ => Some(ts),
            };
            latest = match latest {
                Some(existing) if existing >= ts => Some(existing),
                _ => Some(ts),
            };
        }

        match record.profit_percent {
            Some(value) if value > 0.0 => {
                wins += 1;
                total_profit += value;
                total_known += 1;
            }
            Some(value) if value < 0.0 => {
                losses += 1;
                total_profit += value;
                total_known += 1;
            }
            Some(_) => {
                neutral += 1;
                total_known += 1;
            }
            None => {
                unknown += 1;
            }
        }
    }

    let total = records.len();
    let win_rate = if total_known > 0 {
        (wins as f64 / total_known as f64) * 100.0
    } else {
        0.0
    };

    let avg_profit = if total_known > 0 {
        total_profit / total_known as f64
    } else {
        0.0
    };

    println!("日志文件: {}", log_path);
    println!("总平仓记录: {}", total);
    println!("可解析盈利空间的记录: {}", total_known);
    println!("  盈利: {}", wins);
    println!("  亏损: {}", losses);
    println!("  持平: {}", neutral);
    println!("  未知: {}", unknown);
    println!("胜率 (盈利 / 可解析): {:.2}%", win_rate);
    println!("平均盈利空间: {:.2}%", avg_profit);

    if let Some(start) = earliest {
        println!("首条平仓时间: {}", start);
    }
    if let Some(end) = latest {
        println!("末条平仓时间: {}", end);
    }

    println!("\n按币种统计（Top 10）:");
    let mut counts: Vec<(&String, &usize)> = symbol_counter.iter().collect();
    counts.sort_by(|a, b| b.1.cmp(a.1));
    for (symbol, count) in counts.into_iter().take(10) {
        println!("  {}: {} 条记录", symbol, count);
    }

    println!("\n最近 5 条平仓记录:");
    for record in records.iter().rev().take(5).rev() {
        let time_str = record
            .timestamp
            .map(|ts| ts.to_string())
            .or_else(|| record.raw_time.clone())
            .unwrap_or_else(|| "未知时间".to_string());
        let profit_str = record
            .profit_percent
            .map(|p| format!("{:.2}%", p))
            .unwrap_or_else(|| "未知".to_string());
        let message = record
            .message
            .clone()
            .unwrap_or_else(|| "无完整内容记录".to_string());
        println!("  [{}] {} -> 盈利空间 {}", time_str, message, profit_str);
    }
}
