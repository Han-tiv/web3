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
use chrono::{DateTime, Utc};
use dotenv::dotenv;
use grammers_client::{Client, Config, Update};
use grammers_session::Session;
use log::{error, info, warn};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;

use rust_trading_bot::{
    binance_client::BinanceClient,
    deepseek_client::{DeepSeekClient, Kline, TechnicalIndicators, TradingSignal},
    exchange_trait::{ExchangeClient, Position},
    key_level_finder::KeyLevelFinder,
    technical_analysis::TechnicalAnalyzer,
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

/// 持倉追蹤資訊
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PositionTracker {
    symbol: String,
    entry_price: f64,
    quantity: f64,
    leverage: u32,
    side: String,
    stop_loss_order_id: Option<String>,
    take_profit_order_id: Option<String>,
    entry_time: DateTime<Utc>,
    last_check_time: DateTime<Utc>,
}

/// 交易信號記錄
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignalRecord {
    timestamp: String,
    signal: String,
    confidence: String,
    reason: String,
    price: f64,
}

/// 交易信號歷史
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

struct IntegratedAITrader {
    telegram_client: Arc<Client>,
    exchange: Arc<BinanceClient>,
    deepseek: Arc<DeepSeekClient>,
    analyzer: Arc<TechnicalAnalyzer>,
    level_finder: Arc<KeyLevelFinder>,

    // 配置
    fund_channel_id: i64,
    alpha_keywords: Vec<String>,
    fomo_keywords: Vec<String>,

    // 交易配置 - 动态范围
    min_position_usdt: f64,    // 最小仓位 1 USDT
    max_position_usdt: f64,    // 最大仓位 3 USDT
    min_leverage: u32,         // 最小杠杆 15x
    max_leverage: u32,         // 最大杠杆 20x

    // 内存管理配置
    max_tracked_coins: usize,  // tracked_coins 最大数量
    coin_ttl_hours: i64,       // 币种追踪过期时间(小时)

    // 状态跟踪
    tracked_coins: Arc<RwLock<HashMap<String, FundAlert>>>,
    position_trackers: Arc<RwLock<HashMap<String, PositionTracker>>>,
    signal_history: Arc<RwLock<SignalHistory>>,
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

            min_position_usdt: 1.0,
            max_position_usdt: 3.0,
            min_leverage: 15,
            max_leverage: 20,

            // 内存管理配置
            max_tracked_coins: 100,  // 最多追踪 100 个币种
            coin_ttl_hours: 24,      // 24 小时后自动过期

            tracked_coins: Arc::new(RwLock::new(HashMap::new())),
            position_trackers: Arc::new(RwLock::new(HashMap::new())),
            signal_history: Arc::new(RwLock::new(SignalHistory::new(30))),
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
        let price: f64 = price_regex.captures(text)?.get(1)?.as_str().parse().ok()?;

        // 提取24H涨跌幅
        let change_regex = Regex::new(r"24H[:：]\s*([+-]?[\d.]+)%").ok()?;
        let change_24h: f64 = change_regex.captures(text)?.get(1)?.as_str().parse().ok()?;

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
        let is_alpha = self
            .alpha_keywords
            .iter()
            .any(|kw| message_lower.contains(kw));

        // 检查FOMO关键词或高涨幅
        let is_fomo = self
            .fomo_keywords
            .iter()
            .any(|kw| message_lower.contains(kw))
            || alert.change_24h > 10.0;

        is_alpha || is_fomo
    }

    /// 更新分类 - 简化版本,让AI自己判断
    fn classify_alert(&self, alert: &mut FundAlert) {
        // 所有资金流入信号都统一标记为 FundInflow
        // 不再根据关键词或涨幅过滤,让AI自己分析判断
        if alert.alert_type != AlertType::FundEscape {
            alert.alert_type = AlertType::FundInflow;
        }
    }

    /// 清理过期的追踪币种 - 防止内存泄漏
    async fn cleanup_tracked_coins(&self) {
        let mut coins = self.tracked_coins.write().await;
        let now = Utc::now();

        // 移除过期的币种 (超过 TTL)
        coins.retain(|coin, alert| {
            let age_hours = (now - alert.timestamp).num_hours();
            if age_hours >= self.coin_ttl_hours {
                info!("🗑️  清理过期币种: {} (已追踪 {} 小时)", coin, age_hours);
                false
            } else {
                true
            }
        });

        // 如果超过最大数量，移除最旧的币种
        if coins.len() > self.max_tracked_coins {
            let mut sorted: Vec<_> = coins.iter().map(|(k, v)| (k.clone(), v.timestamp)).collect();
            sorted.sort_by_key(|(_, timestamp)| *timestamp);

            let to_remove = coins.len() - self.max_tracked_coins;
            let coins_to_remove: Vec<String> = sorted.iter().take(to_remove).map(|(coin, _)| coin.clone()).collect();

            for coin in coins_to_remove {
                info!("🗑️  清理超量币种: {} (保持在 {} 个以内)", coin, self.max_tracked_coins);
                coins.remove(&coin);
            }
        }

        if !coins.is_empty() {
            info!("📊 当前追踪币种数: {}/{}", coins.len(), self.max_tracked_coins);
        }
    }

    /// 处理新消息 - 所有信号(包括出逃)都送给AI判断
    async fn handle_message(&self, text: &str) -> Result<()> {
        // 解析资金异动
        if let Some(mut alert) = self.parse_fund_alert(text) {
            // 更新分类
            self.classify_alert(&mut alert);

            let signal_desc = match alert.alert_type {
                AlertType::FundEscape => "⚠️  主力出逃",
                _ => "📊 资金流入",
            };

            info!(
                "\n{}: {} 💰",
                signal_desc, alert.coin
            );
            info!(
                "   价格: ${:.4} | 24H: {:+.2}% | 类型: {}",
                alert.price, alert.change_24h, alert.fund_type
            );

            // 先清理过期数据
            self.cleanup_tracked_coins().await;

            // 保存到跟踪列表
            let mut coins = self.tracked_coins.write().await;
            coins.insert(alert.coin.clone(), alert.clone());
            drop(coins);

            // 触发AI分析(包括出逃信号)
            self.analyze_and_trade(alert).await?;
        }

        Ok(())
    }

    /// 检查是否应该因频繁交易而跳过执行
    fn check_frequent_trading(
        signal: &TradingSignal,
        current_position: Option<&Position>,
        signal_history: &SignalHistory,
    ) -> bool {
        // 如果是 HOLD 信号，直接返回
        if signal.signal == "HOLD" {
            return false;
        }

        // 如果当前有持仓，检查是否反向信号
        if let Some(pos) = current_position {
            let is_reverse_signal = (pos.side == "LONG" && signal.signal == "SELL")
                || (pos.side == "SHORT" && signal.signal == "BUY");

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

    /// 持仓监控线程 - 4小时超时止损 + 分级止盈 + 内存管理
    async fn monitor_positions(self: Arc<Self>) {
        info!("🔍 持仓监控线程已启动");

        let mut cleanup_counter = 0;

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await; // 每分钟检查一次

            cleanup_counter += 1;

            // 每 60 分钟(1小时)执行一次全局清理
            if cleanup_counter >= 60 {
                info!("⏰ 开始执行定期内存清理...");
                self.cleanup_tracked_coins().await;
                self.cleanup_orphaned_trackers().await;
                cleanup_counter = 0;
                info!("✅ 定期内存清理完成");
            }

            let mut trackers = self.position_trackers.write().await;
            let mut to_remove = Vec::new();

            for (symbol, tracker) in trackers.iter_mut() {
                // 更新最后检查时间
                tracker.last_check_time = Utc::now();

                // 获取当前持仓
                let positions = match self.exchange.get_positions().await {
                    Ok(pos) => pos.into_iter().filter(|p| p.symbol == *symbol).collect::<Vec<_>>(),
                    Err(e) => {
                        warn!("⚠️  获取{}持仓失败: {}", symbol, e);
                        continue;
                    }
                };

                // 如果持仓不存在,说明已被止损/止盈触发
                if positions.is_empty() {
                    info!("✅ {} 持仓已平仓(止损/止盈触发)", symbol);
                    to_remove.push(symbol.clone());
                    continue;
                }

                let position = &positions[0];
                let current_price = position.mark_price;
                let entry_price = tracker.entry_price;

                // 计算持仓时长(小时)
                let duration = (Utc::now() - tracker.entry_time).num_minutes() as f64 / 60.0;

                // 计算收益率
                let profit_pct = if tracker.side == "LONG" {
                    ((current_price - entry_price) / entry_price) * 100.0
                } else {
                    ((entry_price - current_price) / entry_price) * 100.0
                };

                info!(
                    "📊 {} 持仓检查: 方向={} | 入场=${:.4} | 当前=${:.4} | 盈亏={:+.2}% | 时长={:.1}h",
                    symbol, tracker.side, entry_price, current_price, profit_pct, duration
                );

                // 【时间止损】4小时未盈利则强制平仓
                if duration >= 4.0 && profit_pct < 1.0 {
                    warn!("⏰ {} 超时4小时且未盈利,执行时间止损", symbol);
                    if let Err(e) = self.close_position_fully(symbol, &tracker.side, tracker.quantity).await {
                        error!("❌ 时间止损失败: {}", e);
                    } else {
                        to_remove.push(symbol.clone());
                    }
                    continue;
                }

                // 【分级止盈】+3%减半仓, +5%清仓
                if profit_pct >= 5.0 {
                    info!("🎯 {} 达到+5%,执行完全止盈", symbol);
                    if let Err(e) = self.close_position_fully(symbol, &tracker.side, tracker.quantity).await {
                        error!("❌ 完全止盈失败: {}", e);
                    } else {
                        to_remove.push(symbol.clone());
                    }
                    continue;
                } else if profit_pct >= 3.0 && tracker.quantity == position.size {
                    // 只在仓位未减半时执行
                    info!("📉 {} 达到+3%,执行减半止盈", symbol);
                    let half_quantity = tracker.quantity / 2.0;
                    if let Err(e) = self.close_position_partially(symbol, &tracker.side, half_quantity).await {
                        error!("❌ 减半止盈失败: {}", e);
                    } else {
                        // 更新tracker的数量
                        tracker.quantity = half_quantity;
                        info!("✅ 已减半仓位,剩余数量: {:.6}", half_quantity);

                        // 移动止损到保本位
                        if let Some(old_sl_id) = &tracker.stop_loss_order_id {
                            let _ = self.exchange.cancel_order(symbol, old_sl_id).await;
                        }
                        match self.exchange.set_stop_loss(symbol, &tracker.side, half_quantity, entry_price).await {
                            Ok(new_sl_id) => {
                                tracker.stop_loss_order_id = Some(new_sl_id);
                                info!("✅ 止损已移动到保本位: ${:.4}", entry_price);
                            }
                            Err(e) => warn!("⚠️  移动止损失败: {}", e),
                        }
                    }
                }
            }

            // 清理已平仓的持仓
            for symbol in to_remove {
                trackers.remove(&symbol);
            }
        }
    }

    /// 清理孤立的持仓追踪器 - 防止内存泄漏
    async fn cleanup_orphaned_trackers(&self) {
        let mut trackers = self.position_trackers.write().await;
        let mut to_remove = Vec::new();

        for (symbol, tracker) in trackers.iter() {
            // 获取实际持仓
            match self.exchange.get_positions().await {
                Ok(positions) => {
                    let has_position = positions.iter().any(|p| p.symbol == *symbol);

                    // 如果没有实际持仓,清理追踪器
                    if !has_position {
                        info!("🗑️  清理孤立追踪器: {} (无对应持仓)", symbol);
                        to_remove.push(symbol.clone());
                    }
                }
                Err(e) => {
                    warn!("⚠️  获取{}持仓失败(清理检查): {}", symbol, e);

                    // 如果超过24小时无法验证,也清理掉
                    let age_hours = (Utc::now() - tracker.last_check_time).num_hours();
                    if age_hours >= 24 {
                        warn!("🗑️  清理陈旧追踪器: {} (超过24小时无法验证)", symbol);
                        to_remove.push(symbol.clone());
                    }
                }
            }
        }

        for symbol in to_remove {
            trackers.remove(&symbol);
        }

        if !trackers.is_empty() {
            info!("📊 当前持仓追踪器数: {}", trackers.len());
        }
    }

    /// 完全平仓
    async fn close_position_fully(&self, symbol: &str, side: &str, quantity: f64) -> Result<()> {
        let close_side = if side == "LONG" { "SELL" } else { "BUY" };

        // 取消现有止损止盈单
        let trackers = self.position_trackers.read().await;
        if let Some(tracker) = trackers.get(symbol) {
            if let Some(sl_id) = &tracker.stop_loss_order_id {
                let _ = self.exchange.cancel_order(symbol, sl_id).await;
            }
            if let Some(tp_id) = &tracker.take_profit_order_id {
                let _ = self.exchange.cancel_order(symbol, tp_id).await;
            }
        }
        drop(trackers);

        // 市价平仓
        self.exchange.market_order(symbol, quantity, close_side).await?;
        info!("✅ {} 已完全平仓", symbol);
        Ok(())
    }

    /// 部分平仓
    async fn close_position_partially(&self, symbol: &str, side: &str, quantity: f64) -> Result<()> {
        let close_side = if side == "LONG" { "SELL" } else { "BUY" };
        self.exchange.market_order(symbol, quantity, close_side).await?;
        info!("✅ {} 已部分平仓: {:.6}", symbol, quantity);
        Ok(())
    }

    /// AI分析并执行交易
    async fn analyze_and_trade(&self, alert: FundAlert) -> Result<()> {
        info!("🧠 开始AI分析: {}", alert.coin);

        // 1. 获取K线数据 - 归一化symbol为BTCUSDT格式
        let symbol = format!("{}USDT", alert.coin);
        info!("🔍 交易对标准化: {} -> {}", alert.coin, symbol);

        // 使用 timeout 避免 API 调用卡死
        let klines_result = tokio::time::timeout(
            tokio::time::Duration::from_secs(10),
            self.exchange.get_klines(&symbol, "15m", Some(100))
        ).await;

        let klines = match klines_result {
            Ok(Ok(data)) => data
                .iter()
                .map(|candle| Kline {
                    timestamp: candle[0] as i64,
                    open: candle[1],
                    high: candle[2],
                    low: candle[3],
                    close: candle[4],
                    volume: candle[5],
                })
                .collect::<Vec<_>>(),
            Ok(Err(e)) => {
                warn!("❌ 获取{}K线失败: {}", symbol, e);
                return Ok(());
            }
            Err(_) => {
                warn!("❌ 获取{}K线超时", symbol);
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
        let prompt =
            self.build_enhanced_prompt(&alert, &klines, &indicators, &key_levels, current_price);

        info!("📝 发送给DeepSeek AI分析...");

        // 5. 调用DeepSeek API分析市场 - 添加超时保护
        let decision_result = tokio::time::timeout(
            tokio::time::Duration::from_secs(30),
            self.deepseek.analyze_market(&prompt)
        ).await;

        let decision = match decision_result {
            Ok(Ok(signal)) => signal,
            Ok(Err(e)) => {
                error!("❌ DeepSeek API调用失败: {}", e);
                info!("💡 Prompt已打印至日志,请检查网络连接或API密钥");
                return Ok(());
            }
            Err(_) => {
                error!("❌ DeepSeek API调用超时(30秒)");
                return Ok(());
            }
        };

        info!("\n📊 DeepSeek AI 决策结果:");
        info!("   信号: {}", decision.signal);
        info!("   置信度: {}", decision.confidence);
        info!("   理由: {}", decision.reason);
        info!("   止损价: ${:.4}", decision.stop_loss);
        info!("   止盈价: ${:.4}", decision.take_profit);

        // 6. 执行交易决策
        if decision.signal == "HOLD" || decision.signal == "SKIP" {
            info!("⏸️  AI建议观望,不执行交易");
            return Ok(());
        }

        // 低信心信号跳过
        if decision.confidence == "LOW" {
            info!("⚠️  置信度较低,跳过交易");
            return Ok(());
        }

        // 6.5 检查当前持仓和防频繁交易
        let current_position = self.exchange.get_positions().await.ok()
            .and_then(|positions| positions.into_iter().find(|p| p.symbol == symbol));

        let signal_history = self.signal_history.read().await;
        if Self::check_frequent_trading(&decision, current_position.as_ref(), &signal_history) {
            info!("⚠️  防频繁交易检查未通过,跳过本次交易");
            return Ok(());
        }
        drop(signal_history);

        // 7. 动态计算仓位和杠杆 - 根据置信度调整
        let (position_usdt, leverage) = match decision.confidence.as_str() {
            "HIGH" => {
                // 高信心: 最大仓位 3U + 最高杠杆 20x = 60U名义价值
                (self.max_position_usdt, self.max_leverage)
            }
            "MEDIUM" => {
                // 中信心: 中等仓位 2U + 中等杠杆 17-18x ≈ 34-36U名义价值
                let mid_position = (self.min_position_usdt + self.max_position_usdt) / 2.0;
                let mid_leverage = (self.min_leverage + self.max_leverage) / 2;
                (mid_position, mid_leverage)
            }
            _ => {
                // 低信心: 最小仓位 1U + 最低杠杆 15x = 15U名义价值 (实际上LOW会被跳过)
                (self.min_position_usdt, self.min_leverage)
            }
        };

        let quantity = position_usdt * leverage as f64 / current_price;

        info!("💰 仓位配置:");
        info!("   投入USDT: {:.2} (动态范围: {:.1}-{:.1}U)",
            position_usdt, self.min_position_usdt, self.max_position_usdt);
        info!("   杠杆倍数: {}x (动态范围: {}-{}x)",
            leverage, self.min_leverage, self.max_leverage);
        info!("   开仓数量: {:.6} {}", quantity, alert.coin);
        info!("   名义价值: {:.2} USDT ({}U × {}x)",
            position_usdt * leverage as f64, position_usdt, leverage);

        // 8. 执行开仓 - 使用动态杠杆
        let side = if decision.signal == "BUY" {
            "LONG"
        } else {
            "SHORT"
        };

        let trade_result = if decision.signal == "BUY" {
            self.exchange
                .open_long(&symbol, quantity, leverage, "CROSSED", false)
                .await
        } else {
            self.exchange
                .open_short(&symbol, quantity, leverage, "CROSSED", false)
                .await
        };

        match trade_result {
            Ok(_) => {
                info!("✅ 交易执行成功!");
                info!("   方向: {}", decision.signal);
                info!("   入场价: ${:.4}", current_price);
                info!("   止损价: ${:.4}", decision.stop_loss);
                info!("   止盈价: ${:.4}", decision.take_profit);

                // 9. 自动设置止损止盈单
                info!("\n🎯 设置自动止损止盈单...");

                // 设置止损单
                let stop_loss_order_id = match self
                    .exchange
                    .set_stop_loss(&symbol, side, quantity, decision.stop_loss)
                    .await
                {
                    Ok(order_id) => {
                        info!("   ✅ 止损单ID: {}", order_id);
                        Some(order_id)
                    }
                    Err(e) => {
                        warn!("   ⚠️  止损单设置失败: {}", e);
                        None
                    }
                };

                // 设置止盈单
                let take_profit_order_id = match self
                    .exchange
                    .set_take_profit(&symbol, side, quantity, decision.take_profit)
                    .await
                {
                    Ok(order_id) => {
                        info!("   ✅ 止盈单ID: {}", order_id);
                        Some(order_id)
                    }
                    Err(e) => {
                        warn!("   ⚠️  止盈单设置失败: {}", e);
                        None
                    }
                };

                // 10. 记录持仓信息到tracker
                let now = Utc::now();
                let tracker = PositionTracker {
                    symbol: symbol.clone(),
                    entry_price: current_price,
                    quantity,
                    leverage,
                    side: side.to_string(),
                    stop_loss_order_id,
                    take_profit_order_id,
                    entry_time: now,
                    last_check_time: now,
                };

                self.position_trackers
                    .write()
                    .await
                    .insert(symbol.clone(), tracker);

                info!("   ✅ 持仓已记录到跟踪器");

                // 11. 记录信号历史
                let signal_record = SignalRecord {
                    timestamp: now.to_rfc3339(),
                    signal: decision.signal.clone(),
                    confidence: decision.confidence.clone(),
                    reason: decision.reason.clone(),
                    price: current_price,
                };

                self.signal_history.write().await.add(signal_record);
                info!("   ✅ 信号已记录到历史");
            }
            Err(e) => {
                error!("❌ 交易执行失败: {}", e);
                error!("   请检查账户余额、API权限或交易对合法性");
            }
        }

        Ok(())
    }

    /// 构建增强的DeepSeek Prompt
    fn build_enhanced_prompt(
        &self,
        alert: &FundAlert,
        klines: &[Kline],
        indicators: &TechnicalIndicators,
        key_levels: &[rust_trading_bot::key_level_finder::KeyLevel],
        current_price: f64,
    ) -> String {
        let alert_type_desc = "📊 主力资金异动信号";

        // 找到最近的关键位
        let (nearest_support, nearest_resistance) = self
            .level_finder
            .find_nearest_levels(key_levels, current_price);

        format!(
            r#"你是专业的日内交易分析师，现在有一个主力资金异动信号需要评估。

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 币种: ${}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

💰 【主力资金信号】
- 信号类型: {}
- 当前价格: ${:.6}
- 24H涨跌幅: {:+.2}%
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

【整合主力关键位策略】
✅ **做多入场条件(BUY)**:
- 价格在支撑位附近(±2%) + 未破位 + RSI<40超卖
- {}
- 资金流入持续、未见主力出逃信号
- 设置好止损位（支撑位-2%）

✅ **做空入场条件(SELL)**:
- 跌破主力支撑位 + 空头排列(SMA5<SMA20<SMA50)
- RSI>40(非超卖区,避免抄底反弹)
- MACD死叉且负值扩大
- 24H跌幅>-5%且趋势延续,或从高位回落>15%
- 设置好止损位（阻力位+2%或前高+2%）

❌ **不入场条件**:
- 做多时: 已大幅拉升(>30%)且无回调、RSI>70严重超买
- 做空时: RSI<30严重超卖(抄底风险)、无明确破位
- 流动性极差、关键位不明确

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
    "signal": "BUY|SELL|HOLD|SKIP",
    "confidence": "HIGH|MEDIUM|LOW",
    "stop_loss": 止损价格(数字),
    "take_profit": 止盈价格(数字),
    "reason": "简要理由(100字以内,含关键位判断+趋势+技术依据)"
}}

【字段说明】
- signal: BUY(强烈推荐做多), SELL(强烈推荐做空), HOLD(观望), SKIP(不推荐)
- confidence: 置信度(HIGH/MEDIUM/LOW)
- stop_loss: 止损价格(做多时:入场价-2%或支撑位-2%; 做空时:入场价+2%或阻力位+2%)
- take_profit: 第一止盈目标(做多时:+3%; 做空时:-3%),系统会自动设置±5%清仓
- reason: 决策理由,必须包含主力关键位状态+趋势判断+技术指标

请综合分析后给出明确决策！
"#,
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
            self.format_key_levels(
                key_levels,
                current_price,
                &nearest_support,
                &nearest_resistance
            ),
            current_price,
            klines.iter().map(|k| k.high).fold(f64::MIN, f64::max),
            klines.iter().map(|k| k.low).fold(f64::MAX, f64::min),
            klines.last().unwrap().volume,
            self.format_entry_condition(&nearest_support, &nearest_resistance, current_price),
        )
    }

    fn get_bb_position(
        &self,
        price: f64,
        indicators: &TechnicalIndicators,
    ) -> &str {
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
    let trader =
        Arc::new(IntegratedAITrader::new(telegram_client, exchange, deepseek_api_key).await);

    // 启动持仓监控线程
    let monitor_trader = trader.clone();
    tokio::spawn(async move {
        monitor_trader.monitor_positions().await;
    });
    info!("✅ 持仓监控线程已启动\n");

    // 解析所有频道实体 - 完整修复 "unknown peer" 问题
    info!("🔍 正在缓存所有频道实体...");

    // 遍历所有对话,缓存所有频道实体,防止 grammers unknown peer 问题
    let mut target_channel_id: Option<i64> = None;
    let mut cached_channels = 0;
    let mut dialogs = trader.telegram_client.iter_dialogs();

    while let Some(dialog) = dialogs.next().await? {
        if let grammers_client::types::Chat::Channel(channel) = dialog.chat() {
            cached_channels += 1;

            // 检查是否为目标频道
            if channel.id() == trader.fund_channel_id {
                info!("✅ 目标频道已解析: {} (ID: {})", channel.title(), channel.id());
                target_channel_id = Some(channel.id());
            }
        }
    }

    info!("✅ 已缓存 {} 个频道实体 (防止消息丢失)", cached_channels);

    let target_channel_id = match target_channel_id {
        Some(id) => id,
        None => {
            anyhow::bail!("❌ 无法找到目标频道 (ID: {}),请确保已加入该频道", trader.fund_channel_id);
        }
    };

    info!("📡 开始实时监控...");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // 实时监控循环
    loop {
        match trader.telegram_client.next_update().await {
            Ok(Update::NewMessage(message)) if !message.outgoing() => {
                match message.chat() {
                    grammers_client::types::Chat::Channel(channel)
                        if channel.id() == target_channel_id =>
                    {
                        let text = message.text();
                        if !text.is_empty() {
                            if let Err(e) = trader.handle_message(text).await {
                                error!("❌ 处理消息错误: {}", e);
                            }
                        }
                    }
                    _ => {}
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
