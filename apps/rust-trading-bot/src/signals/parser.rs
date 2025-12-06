// 消息解析器模块
use anyhow::Result;
use chrono::{Duration, Utc};
use log::{debug, info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::types::{AlertType, FundAlert};

// 注意：IntegratedAITrader的定义在bin/integrated_ai_trader.rs中
// 这里使用泛型trait来解耦

/// 信号上下文 trait - 定义消息处理所需的接口
#[async_trait::async_trait]
pub trait SignalContext: Send + Sync {
    fn tracked_coins(&self) -> Arc<RwLock<HashMap<String, FundAlert>>>;
    fn coin_ttl_hours(&self) -> i64;
    async fn analyze_and_trade(&self, alert: FundAlert) -> Result<()>;
    async fn get_current_price(&self, symbol: &str) -> Result<f64>;
}

/// 消息解析器 - 静态方法集合
pub struct MessageParser;

impl MessageParser {
    /// 清理过期追踪币种
    pub async fn cleanup_tracked_coins<T: SignalContext>(trader: &T) {
        let _now = Utc::now();
        let _ttl = Duration::hours(trader.coin_ttl_hours());
        
        // 先获取Arc<RwLock>，避免临时值被释放
        let tracked_coins = trader.tracked_coins();
        let mut tracked = tracked_coins.write().await;
        
        let before_count = tracked.len();
        
        // 清理逻辑：移除空消息或过期的币种
        // 注意：FundAlert没有timestamp字段，这里基于raw_message非空来保留
        // 真正的TTL清理需要在FundAlert中添加timestamp字段
        tracked.retain(|symbol, alert| {
            // 保留有效的信号
            if alert.raw_message.is_empty() {
                debug!("🗑️ 清理空消息币种: {}", symbol);
                false
            } else {
                true
            }
        });
        
        let after_count = tracked.len();
        let removed = before_count - after_count;
        
        drop(tracked);
        
        if removed > 0 {
            info!("🧹 清理追踪币种: 移除{}个, 剩余{}, TTL={}h", removed, after_count, trader.coin_ttl_hours());
        } else {
            debug!("🧹 追踪币种清理完成: 无需移除, 当前{}个", after_count);
        }
    }
    
    /// 处理消息 - 解析Telegram文本消息并创建交易信号
    pub async fn handle_message<T: SignalContext>(trader: &T, text: &str) -> Result<()> {
        debug!("📨 收到消息: {}", text.chars().take(100).collect::<String>());
        
        // 基础消息解析逻辑
        // 1. 检查是否包含资金流入/流出关键词
        let is_inflow = text.contains("流入") || text.contains("Inflow") || text.contains("资金异动");
        let is_outflow = text.contains("流出") || text.contains("Outflow") || text.contains("出逃");
        
        if !is_inflow && !is_outflow {
            debug!("⏭️ 跳过非资金信号消息");
            return Ok(());
        }
        
        // 2. 尝试提取币种符号 (简化版本，实际应使用coin_parser)
        let symbol = extract_symbol_from_message(text);
        if symbol.is_empty() {
            warn!("⚠️ 无法从消息中提取币种符号");
            return Ok(());
        }
        
        // 3. 提取价格信息 (如果有)
        let price = extract_price_from_message(text).unwrap_or(0.0);
        
        // 4. 创建FundAlert
        let alert = FundAlert {
            coin: symbol.clone(),
            raw_message: text.to_string(),
            change_24h: 0.0, // 需要从消息中提取或API获取
            alert_type: if is_inflow { AlertType::Inflow } else { AlertType::Outflow },
            fund_type: if is_inflow { "资金流入".to_string() } else { "资金流出".to_string() },
            price,
        };
        
        info!("📊 解析信号: {} | 类型:{} | 价格:{:.4}", symbol, alert.fund_type, price);
        
        // 5. 触发交易分析
        trader.analyze_and_trade(alert).await
    }
    
    /// 处理Valuescan消息
    pub async fn handle_valuescan_message<T: SignalContext>(
        trader: &T,
        symbol: &str,
        message_text: &str,
        score: i32,
        signal_type: &str,
    ) -> Result<()> {
        info!("📊 Valuescan信号: {} | 评分:{} | 类型:{}", symbol, score, signal_type);
        
        // 获取当前价格
        let current_price = trader.get_current_price(symbol).await.unwrap_or(0.0);
        
        // 创建FundAlert
        let alert = FundAlert {
            coin: symbol.replace("USDT", "").to_string(),
            raw_message: message_text.to_string(),
            change_24h: 0.0,
            alert_type: if signal_type.contains("流入") || signal_type.contains("Inflow") {
                AlertType::Inflow
            } else {
                AlertType::Outflow
            },
            fund_type: signal_type.to_string(),
            price: current_price,
        };
        
        // 调用交易分析
        trader.analyze_and_trade(alert).await
    }
    
    /// 处理收到的信号
    pub async fn handle_incoming_alert<T: SignalContext>(
        trader: &T,
        alert: FundAlert,
        _raw_message: &str,
        _persist_signal: bool,
    ) -> Result<()> {
        trader.analyze_and_trade(alert).await
    }
    
    /// 处理分类后的信号
    pub async fn process_classified_alert<T: SignalContext>(
        trader: &T,
        alert: FundAlert,
    ) -> Result<()> {
        trader.analyze_and_trade(alert).await
    }
}

// ============ 辅助函数 ============

/// 从消息中提取币种符号
fn extract_symbol_from_message(text: &str) -> String {
    // 简化版本：查找常见模式如 $BTC, BTC/USDT, BTCUSDT等
    if let Some(pos) = text.find('$') {
        let after_dollar = &text[pos+1..];
        if let Some(end) = after_dollar.find(|c: char| !c.is_alphanumeric()) {
            return after_dollar[..end].to_uppercase();
        }
    }
    
    // 查找 XXXUSDT 模式
    for word in text.split_whitespace() {
        if word.ends_with("USDT") && word.len() > 4 {
            return word[..word.len()-4].to_uppercase();
        }
    }
    
    String::new()
}

/// 从消息中提取价格
fn extract_price_from_message(text: &str) -> Option<f64> {
    // 查找价格模式：$123.45 或 价格: 123.45
    use regex::Regex;
    let price_regex = Regex::new(r"[\$价格:：]\s*([0-9]+\.?[0-9]*)").ok()?;
    if let Some(cap) = price_regex.captures(text) {
        return cap.get(1)?.as_str().parse().ok();
    }
    None
}
