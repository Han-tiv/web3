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

use rust_trading_bot::support_analyzer::{Kline as SupportKline, SupportAnalyzer};
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

/// 持仓监控阶段需要执行的动作，采用“先收集再处理”策略避免锁重入
#[derive(Debug)]
enum PositionAction {
    FullClose {
        symbol: String,
        side: String,
        quantity: f64,
        reason: String,
    },
    PartialClose {
        symbol: String,
        side: String,
        close_quantity: f64,
        close_pct: f64,
        entry_price: f64,
        remaining_quantity: f64,
        stop_loss_order_id: Option<String>,
    },
    Remove(String),
    SetLimitOrder {
        symbol: String,
        side: String,
        quantity: f64,
        limit_price: f64,
        take_profit_order_id: Option<String>,
    },
}

/// 对追踪器的更新操作，统一在短暂写锁中落盘
#[derive(Debug)]
enum TrackerMutation {
    QuantityAndStopLoss {
        symbol: String,
        new_quantity: f64,
        new_stop_loss_order_id: Option<String>,
    },
    TakeProfitOrder {
        symbol: String,
        new_take_profit_order_id: Option<String>,
    },
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
    min_position_usdt: f64, // 最小仓位 1 USDT
    max_position_usdt: f64, // 最大仓位 2 USDT
    min_leverage: u32,      // 最小杠杆 6x
    max_leverage: u32,      // 最大杠杆 10x

    // 内存管理配置
    max_tracked_coins: usize, // tracked_coins 最大数量
    coin_ttl_hours: i64,      // 币种追踪过期时间(小时)

    // 状态跟踪
    tracked_coins: Arc<RwLock<HashMap<String, FundAlert>>>,
    position_trackers: Arc<RwLock<HashMap<String, PositionTracker>>>,
    signal_history: Arc<RwLock<SignalHistory>>,
    last_analysis_time: Arc<RwLock<HashMap<String, DateTime<Utc>>>>, // 【优化1】信号去重
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
            max_position_usdt: 2.0,
            min_leverage: 6,
            max_leverage: 10,

            // 内存管理配置
            max_tracked_coins: 100, // 最多追踪 100 个币种
            coin_ttl_hours: 24,     // 24 小时后自动过期

            tracked_coins: Arc::new(RwLock::new(HashMap::new())),
            position_trackers: Arc::new(RwLock::new(HashMap::new())),
            signal_history: Arc::new(RwLock::new(SignalHistory::new(30))),
            last_analysis_time: Arc::new(RwLock::new(HashMap::new())), // 【优化1】初始化去重map
        }
    }

    /// 解析资金异动消息
    fn parse_fund_alert(&self, text: &str) -> Option<FundAlert> {
        // 提取币种 $COIN格式
        let coin_regex = Regex::new(r"\$([A-Z0-9]+)").ok()?;
        let coin = coin_regex.captures(text)?.get(1)?.as_str().to_string();

        // 判断消息类型 - 支持【资金异动】和【Alpha】格式
        let alert_type = if text.contains("出逃") || text.contains("撤离") {
            AlertType::FundEscape
        } else if text.contains("【资金异动】")
            || text.contains("【Alpha】")
            || text.contains("【FOMO】")
        {
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
            let mut sorted: Vec<_> = coins
                .iter()
                .map(|(k, v)| (k.clone(), v.timestamp))
                .collect();
            sorted.sort_by_key(|(_, timestamp)| *timestamp);

            let to_remove = coins.len() - self.max_tracked_coins;
            let coins_to_remove: Vec<String> = sorted
                .iter()
                .take(to_remove)
                .map(|(coin, _)| coin.clone())
                .collect();

            for coin in coins_to_remove {
                info!(
                    "🗑️  清理超量币种: {} (保持在 {} 个以内)",
                    coin, self.max_tracked_coins
                );
                coins.remove(&coin);
            }
        }

        if !coins.is_empty() {
            info!(
                "📊 当前追踪币种数: {}/{}",
                coins.len(),
                self.max_tracked_coins
            );
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

            info!("\n{}: {} 💰", signal_desc, alert.coin);
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
            tokio::time::sleep(tokio::time::Duration::from_secs(300)).await; // 每5分钟检查一次

            cleanup_counter += 1;

            // 每 12 次检查(60分钟)执行一次全局清理
            if cleanup_counter >= 12 {
                info!("⏰ 开始执行定期内存清理...");
                self.cleanup_tracked_coins().await;
                self.cleanup_orphaned_trackers().await;
                cleanup_counter = 0;
                info!("✅ 定期内存清理完成");
            }

            #[derive(Clone)]
            struct TrackerSnapshot {
                symbol: String,
                side: String,
                quantity: f64,
                entry_price: f64,
                entry_time: DateTime<Utc>,
                leverage: u32,
                stop_loss_order_id: Option<String>,
                take_profit_order_id: Option<String>,
            }

            let now = Utc::now();
            let tracker_snapshots: HashMap<String, TrackerSnapshot> = {
                let mut trackers = self.position_trackers.write().await;
                trackers
                    .iter_mut()
                    .map(|(symbol, tracker)| {
                        tracker.last_check_time = now;
                        (
                            symbol.clone(),
                            TrackerSnapshot {
                                symbol: symbol.clone(),
                                side: tracker.side.clone(),
                                quantity: tracker.quantity,
                                entry_price: tracker.entry_price,
                                entry_time: tracker.entry_time,
                                leverage: tracker.leverage,
                                stop_loss_order_id: tracker.stop_loss_order_id.clone(),
                                take_profit_order_id: tracker.take_profit_order_id.clone(),
                            },
                        )
                    })
                    .collect()
            };

            if tracker_snapshots.is_empty() {
                continue;
            }

            let mut actions_to_execute = Vec::new();

            for snapshot in tracker_snapshots.values() {
                let symbol = snapshot.symbol.clone();
                let side = snapshot.side.clone();
                let entry_price = snapshot.entry_price;
                let entry_time = snapshot.entry_time;
                let quantity = snapshot.quantity;

                // 获取当前持仓
                let positions = match self.exchange.get_positions().await {
                    Ok(pos) => pos
                        .into_iter()
                        .filter(|p| p.symbol == symbol)
                        .collect::<Vec<_>>(),
                    Err(e) => {
                        warn!("⚠️  获取{}持仓失败: {}", symbol, e);
                        warn!("🔍 错误详情: {:?}", e);
                        continue;
                    }
                };

                // 如果持仓不存在,说明已被止损/止盈触发
                if positions.is_empty() {
                    info!("✅ {} 持仓已平仓(止损/止盈触发)", symbol);
                    actions_to_execute.push(PositionAction::Remove(symbol));
                    continue;
                }

                let position = &positions[0];
                let current_price = position.mark_price;

                // 计算持仓时长(小时)
                let duration = (Utc::now() - entry_time).num_minutes() as f64 / 60.0;

                // 计算收益率
                let profit_pct = if side == "LONG" {
                    ((current_price - entry_price) / entry_price) * 100.0
                } else {
                    ((entry_price - current_price) / entry_price) * 100.0
                };

                info!(
                    "📊 {} 持仓检查: 方向={} | 入场=${:.4} | 当前=${:.4} | 盈亏={:+.2}% | 时长={:.1}h",
                    symbol, side, entry_price, current_price, profit_pct, duration
                );

                // 【时间止损】4小时未盈利则强制平仓
                if duration >= 4.0 && profit_pct < 1.0 {
                    warn!("⏰ {} 超时4小时且未盈利,执行时间止损", symbol);
                    actions_to_execute.push(PositionAction::FullClose {
                        symbol,
                        side,
                        quantity,
                        reason: "timeout".to_string(),
                    });
                    continue;
                }

                // 【AI 动态止盈评估】对所有持仓调用 AI, 取代固定 +3%/+5% 止盈
                info!(
                    "🤖 {} 当前盈亏 {:+.2}%, 调用 AI 评估持仓管理...",
                    snapshot.symbol, profit_pct
                );

                // 获取多周期K线数据 (5m, 15m, 1h)
                let (klines_5m_result, klines_15m_result, klines_1h_result) = tokio::join!(
                    tokio::time::timeout(
                        tokio::time::Duration::from_secs(10),
                        self.exchange.get_klines(&snapshot.symbol, "5m", Some(50))
                    ),
                    tokio::time::timeout(
                        tokio::time::Duration::from_secs(10),
                        self.exchange.get_klines(&snapshot.symbol, "15m", Some(100))
                    ),
                    tokio::time::timeout(
                        tokio::time::Duration::from_secs(10),
                        self.exchange.get_klines(&snapshot.symbol, "1h", Some(48))
                    )
                );

                // 解析5m K线
                let klines_5m = match klines_5m_result {
                    Ok(Ok(data)) => data
                        .iter()
                        .map(|candle| rust_trading_bot::deepseek_client::Kline {
                            timestamp: candle[0] as i64,
                            open: candle[1],
                            high: candle[2],
                            low: candle[3],
                            close: candle[4],
                            volume: candle[5],
                        })
                        .collect::<Vec<_>>(),
                    Ok(Err(e)) => {
                        warn!("⚠️  获取{}5mK线失败: {}, 跳过AI评估", snapshot.symbol, e);
                        continue;
                    }
                    Err(_) => {
                        warn!("⚠️  获取{}5mK线超时, 跳过AI评估", snapshot.symbol);
                        continue;
                    }
                };

                // 解析15m K线
                let klines = match klines_15m_result {
                    Ok(Ok(data)) => data
                        .iter()
                        .map(|candle| rust_trading_bot::deepseek_client::Kline {
                            timestamp: candle[0] as i64,
                            open: candle[1],
                            high: candle[2],
                            low: candle[3],
                            close: candle[4],
                            volume: candle[5],
                        })
                        .collect::<Vec<_>>(),
                    Ok(Err(e)) => {
                        warn!("⚠️  获取{}15mK线失败: {}, 跳过AI评估", snapshot.symbol, e);
                        continue;
                    }
                    Err(_) => {
                        warn!("⚠️  获取{}15mK线超时, 跳过AI评估", snapshot.symbol);
                        continue;
                    }
                };

                // 解析1h K线
                let klines_1h = match klines_1h_result {
                    Ok(Ok(data)) => data
                        .iter()
                        .map(|candle| rust_trading_bot::deepseek_client::Kline {
                            timestamp: candle[0] as i64,
                            open: candle[1],
                            high: candle[2],
                            low: candle[3],
                            close: candle[4],
                            volume: candle[5],
                        })
                        .collect::<Vec<_>>(),
                    Ok(Err(e)) => {
                        warn!("⚠️  获取{}1hK线失败: {}, 跳过AI评估", snapshot.symbol, e);
                        continue;
                    }
                    Err(_) => {
                        warn!("⚠️  获取{}1hK线超时, 跳过AI评估", snapshot.symbol);
                        continue;
                    }
                };

                if klines.len() < 20 {
                    warn!(
                        "⚠️  K线数据不足: {} (需要至少20根), 跳过AI评估",
                        klines.len()
                    );
                    continue;
                }

                // 计算技术指标 (基于15m)
                let indicators = self.analyzer.calculate_indicators(&klines);

                // 方案2支撑位分析 + 三周期数据转换
                let convert_to_support_klines = |source: &[Kline]| -> Vec<SupportKline> {
                    source
                        .iter()
                        .map(|k| SupportKline {
                            open: k.open,
                            high: k.high,
                            low: k.low,
                            close: k.close,
                            volume: k.volume,
                        })
                        .collect()
                };

                let support_klines_5m = convert_to_support_klines(&klines_5m);
                let support_klines_15m = convert_to_support_klines(&klines);
                let support_klines_1h = convert_to_support_klines(&klines_1h);

                let support_analyzer = SupportAnalyzer::new();
                let support_analysis = match support_analyzer.analyze_supports(
                    &support_klines_5m,
                    &support_klines_15m,
                    &support_klines_1h,
                    current_price,
                    entry_price,
                    indicators.sma_20,
                    indicators.sma_50,
                    indicators.bb_lower,
                    indicators.bb_middle,
                ) {
                    Ok(analysis) => analysis,
                    Err(e) => {
                        warn!("⚠️  {} 支撑位分析失败: {}", snapshot.symbol, e);
                        continue;
                    }
                };
                let support_text = support_analyzer.format_support_analysis(&support_analysis);

                let last_5m_close = klines_5m.last().unwrap().close;
                let deviation = ((current_price - last_5m_close) / last_5m_close) * 100.0;
                let deviation_desc = if deviation.abs() < 0.5 {
                    format!("价格稳定 ({:+.2}%)", deviation)
                } else if deviation > 1.0 {
                    format!("正在形成的5m K线继续上涨 {:+.2}% ✅", deviation)
                } else if deviation < -1.0 {
                    format!("正在形成的5m K线继续下跌 {:+.2}% ⚠️", deviation)
                } else {
                    format!("轻微波动 ({:+.2}%)", deviation)
                };

                // 构建持仓管理 prompt - 传入三个周期的K线
                let prompt = self.deepseek.build_position_management_prompt(
                    &snapshot.symbol,
                    &side,
                    entry_price,
                    current_price,
                    profit_pct,
                    duration,
                    &klines_5m,
                    &klines,
                    &klines_1h,
                    &indicators,
                    &support_text,
                    &deviation_desc,
                );

                // 调用 AI 分析
                let ai_decision_result = tokio::time::timeout(
                    tokio::time::Duration::from_secs(30),
                    self.deepseek.analyze_position_management(&prompt),
                )
                .await;

                let ai_decision = match ai_decision_result {
                    Ok(Ok(decision)) => decision,
                    Ok(Err(e)) => {
                        error!("❌ AI持仓评估失败: {}, 保持持仓", e);
                        continue;
                    }
                    Err(_) => {
                        warn!("⚠️  AI持仓评估超时, 保持持仓");
                        continue;
                    }
                };

                info!(
                    "🎯 AI 决策: {} | 理由: {} | 盈利潜力: {} | 置信度: {}",
                    ai_decision.action,
                    ai_decision.reason,
                    ai_decision.profit_potential,
                    ai_decision.confidence
                );

                // 根据 AI 决策执行操作
                match ai_decision.action.as_str() {
                    "HOLD" => {
                        info!("✅ AI 建议继续持有 {}", snapshot.symbol);
                    }
                    "PARTIAL_CLOSE" => {
                        if let Some(close_pct) = ai_decision.close_percentage {
                            info!("📉 AI 建议部分平仓 {} ({}%)", snapshot.symbol, close_pct);
                            let close_quantity =
                                (quantity * (close_pct / 100.0)).clamp(0.0, quantity);
                            let remaining_quantity = (quantity - close_quantity).max(0.0);

                            if close_quantity <= f64::EPSILON {
                                warn!("⚠️  计算得到的平仓数量过小, 跳过本次部分平仓");
                                continue;
                            }

                            actions_to_execute.push(PositionAction::PartialClose {
                                symbol: snapshot.symbol.clone(),
                                side,
                                close_quantity,
                                close_pct,
                                entry_price,
                                remaining_quantity,
                                stop_loss_order_id: snapshot.stop_loss_order_id.clone(),
                            });
                        } else {
                            warn!("⚠️  AI 建议部分平仓但未提供百分比,保持持仓");
                        }
                    }
                    "FULL_CLOSE" => {
                        info!("🚨 AI 建议全部平仓 {}", snapshot.symbol);
                        actions_to_execute.push(PositionAction::FullClose {
                            symbol: snapshot.symbol.clone(),
                            side,
                            quantity,
                            reason: "ai_decision".to_string(),
                        });
                    }
                    "SET_LIMIT_ORDER" => {
                        if let Some(limit_price) = ai_decision.limit_price {
                            info!(
                                "🎯 AI 建议设置限价止盈单 {} @ ${:.4}",
                                snapshot.symbol, limit_price
                            );
                            actions_to_execute.push(PositionAction::SetLimitOrder {
                                symbol: snapshot.symbol.clone(),
                                side,
                                quantity,
                                limit_price,
                                take_profit_order_id: snapshot.take_profit_order_id.clone(),
                            });
                        } else {
                            warn!("⚠️  AI 建议设置限价单但未提供价格,保持持仓");
                        }
                    }
                    _ => {
                        warn!("⚠️  未知的 AI 决策动作: {}, 保持持仓", ai_decision.action);
                    }
                }
            }

            if actions_to_execute.is_empty() {
                continue;
            }

            let mut tracker_mutations = Vec::new();
            let mut symbols_to_remove = Vec::new();

            for action in actions_to_execute {
                match action {
                    PositionAction::FullClose {
                        symbol,
                        side,
                        quantity,
                        reason,
                    } => {
                        if let Err(e) = self.close_position_fully(&symbol, &side, quantity).await {
                            error!("❌ 全部平仓失败({}): {}", reason, e);
                        } else {
                            symbols_to_remove.push(symbol);
                        }
                    }
                    PositionAction::PartialClose {
                        symbol,
                        side,
                        close_quantity,
                        close_pct,
                        entry_price,
                        remaining_quantity,
                        stop_loss_order_id,
                    } => {
                        if let Err(e) = self
                            .close_position_partially(&symbol, &side, close_quantity)
                            .await
                        {
                            error!("❌ 部分平仓失败: {}", e);
                            continue;
                        }

                        info!(
                            "✅ 已平仓 {:.2}%, 剩余数量: {:.6}",
                            close_pct, remaining_quantity
                        );

                        if let Some(order_id) = stop_loss_order_id {
                            let _ = self.exchange.cancel_order(&symbol, &order_id).await;
                        }

                        if remaining_quantity > f64::EPSILON {
                            match self
                                .exchange
                                .set_stop_loss(&symbol, &side, remaining_quantity, entry_price)
                                .await
                            {
                                Ok(new_sl_id) => {
                                    tracker_mutations.push(TrackerMutation::QuantityAndStopLoss {
                                        symbol,
                                        new_quantity: remaining_quantity,
                                        new_stop_loss_order_id: Some(new_sl_id),
                                    });
                                    info!("✅ 止损已移动到保本位: ${:.4}", entry_price);
                                }
                                Err(e) => {
                                    warn!("⚠️  移动止损失败: {}", e);
                                    tracker_mutations.push(TrackerMutation::QuantityAndStopLoss {
                                        symbol,
                                        new_quantity: remaining_quantity,
                                        new_stop_loss_order_id: None,
                                    });
                                }
                            }
                        } else {
                            symbols_to_remove.push(symbol);
                        }
                    }
                    PositionAction::SetLimitOrder {
                        symbol,
                        side,
                        quantity,
                        limit_price,
                        take_profit_order_id,
                    } => {
                        if let Some(order_id) = take_profit_order_id {
                            let _ = self.exchange.cancel_order(&symbol, &order_id).await;
                        }

                        match self
                            .exchange
                            .set_limit_take_profit(&symbol, &side, quantity, limit_price)
                            .await
                        {
                            Ok(order_id) => {
                                tracker_mutations.push(TrackerMutation::TakeProfitOrder {
                                    symbol,
                                    new_take_profit_order_id: Some(order_id),
                                });
                                info!("✅ 限价止盈单已设置 @ ${:.4}", limit_price);
                            }
                            Err(e) => {
                                error!("❌ 设置限价止盈单失败: {}", e);
                            }
                        }
                    }
                    PositionAction::Remove(symbol) => {
                        symbols_to_remove.push(symbol);
                    }
                }
            }

            if !tracker_mutations.is_empty() || !symbols_to_remove.is_empty() {
                let mut trackers = self.position_trackers.write().await;

                for mutation in tracker_mutations {
                    match mutation {
                        TrackerMutation::QuantityAndStopLoss {
                            symbol,
                            new_quantity,
                            new_stop_loss_order_id,
                        } => {
                            if let Some(tracker) = trackers.get_mut(&symbol) {
                                tracker.quantity = new_quantity;
                                tracker.stop_loss_order_id = new_stop_loss_order_id;
                            }
                        }
                        TrackerMutation::TakeProfitOrder {
                            symbol,
                            new_take_profit_order_id,
                        } => {
                            if let Some(tracker) = trackers.get_mut(&symbol) {
                                tracker.take_profit_order_id = new_take_profit_order_id;
                            }
                        }
                    }
                }

                for symbol in symbols_to_remove {
                    trackers.remove(&symbol);
                }
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
                    warn!("🔍 错误详情: {:?}", e);

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

        // 使用限价单平仓，稍微穿透当前价确保成交
        let current_price = self.exchange.get_current_price(symbol).await?;
        let position_side = if side == "LONG" { "LONG" } else { "SHORT" };
        let limit_price = if side == "LONG" {
            current_price * 0.999
        } else {
            current_price * 1.001
        };
        let order_id = self
            .exchange
            .limit_order(
                symbol,
                quantity,
                close_side,
                limit_price,
                Some(position_side),
            )
            .await?;
        info!(
            "✅ {} 已完全平仓，限价: {:.4}，订单ID: {}",
            symbol, limit_price, order_id
        );
        Ok(())
    }

    /// 部分平仓
    async fn close_position_partially(
        &self,
        symbol: &str,
        side: &str,
        quantity: f64,
    ) -> Result<()> {
        let close_side = if side == "LONG" { "SELL" } else { "BUY" };
        let current_price = self.exchange.get_current_price(symbol).await?;
        let position_side = if side == "LONG" { "LONG" } else { "SHORT" };
        let limit_price = if side == "LONG" {
            current_price * 0.999
        } else {
            current_price * 1.001
        };
        let order_id = self
            .exchange
            .limit_order(
                symbol,
                quantity,
                close_side,
                limit_price,
                Some(position_side),
            )
            .await?;
        info!(
            "✅ {} 已部分平仓: {:.6}，限价: {:.4}，订单ID: {}",
            symbol, quantity, limit_price, order_id
        );
        Ok(())
    }

    /// AI分析并执行交易
    async fn analyze_and_trade(&self, alert: FundAlert) -> Result<()> {
        info!("🧠 开始AI分析: {}", alert.coin);

        // 【优化1: 信号去重】检查30秒内是否已分析过该币种
        let mut last_times = self.last_analysis_time.write().await;
        if let Some(last_time) = last_times.get(&alert.coin) {
            let elapsed = (Utc::now() - *last_time).num_seconds();
            if elapsed < 30 {
                info!("⏭️  跳过重复分析: {} ({}秒前已分析)", alert.coin, elapsed);
                return Ok(());
            }
        }
        last_times.insert(alert.coin.clone(), Utc::now());
        drop(last_times);

        // 1. 获取K线数据 - 归一化symbol为BTCUSDT格式
        let symbol = format!("{}USDT", alert.coin);
        info!("🔍 交易对标准化: {} -> {}", alert.coin, symbol);

        // 1.5 获取历史表现 - 12小时内交易记录
        let perf_opt = match self.exchange.get_symbol_performance(&symbol, 12).await {
            Ok(p) => p,
            Err(e) => {
                warn!("⚠️  获取{}历史表现失败: {}", symbol, e);
                None
            }
        };

        // 构建历史表现提示
        let history_prompt = if let Some(perf) = &perf_opt {
            use rust_trading_bot::binance_client::{BinanceClient, RiskLevel};
            let risk_level = BinanceClient::get_risk_level(perf);

            info!(
                "📊 {} 历史表现(12h): {}笔交易, 胜率{:.1}%, 保证金收益率{:+.2}%, 风险等级:{:?}",
                symbol, perf.trade_count, perf.win_rate, perf.margin_loss_rate, risk_level
            );

            match risk_level {
                RiskLevel::High => format!(
                    "\n\n⚠️ 【风险警告】该币种近12小时表现糟糕：\n\
                    - 保证金亏损率: {:.2}% (严重亏损)\n\
                    - 交易次数: {}笔\n\
                    - 胜率: {:.1}%\n\
                    - 总盈亏: {:.4} USDT\n\n\
                    ⛔ 建议：该币种历史表现极差,强烈建议SKIP或降低置信度至LOW。\n\
                    除非有压倒性的技术优势(如明显支撑位+异动首次出现),否则不做。",
                    perf.margin_loss_rate, perf.trade_count, perf.win_rate, perf.total_pnl
                ),
                RiskLevel::Medium => format!(
                    "\n\n⚠️ 【谨慎提示】该币种近12小时表现不佳：\n\
                    - 保证金亏损率: {:.2}%\n\
                    - 交易次数: {}笔\n\
                    - 胜率: {:.1}%\n\
                    - 总盈亏: {:.4} USDT\n\n\
                    建议：提高决策标准,需要更强的技术信号才能开仓。信心度建议MEDIUM或以下。",
                    perf.margin_loss_rate, perf.trade_count, perf.win_rate, perf.total_pnl
                ),
                RiskLevel::Low => format!(
                    "\n\n📉 【轻度负面】该币种近12小时表现一般：\n\
                    - 保证金亏损率: {:.2}%\n\
                    - 交易次数: {}笔\n\
                    - 胜率: {:.1}%\n\
                    - 总盈亏: {:.4} USDT\n\n\
                    建议：略微提高警惕,按正常标准决策即可。",
                    perf.margin_loss_rate, perf.trade_count, perf.win_rate, perf.total_pnl
                ),
                RiskLevel::Normal => {
                    if perf.margin_loss_rate > 10.0 {
                        format!(
                            "\n\n✅ 【正面参考】该币种近12小时表现优秀：\n\
                            - 保证金收益率: +{:.2}%\n\
                            - 交易次数: {}笔\n\
                            - 胜率: {:.1}%\n\
                            - 总盈亏: +{:.4} USDT\n\n\
                            建议：该币种历史盈利,可以适当提高信心,但仍需结合技术面判断。",
                            perf.margin_loss_rate, perf.trade_count, perf.win_rate, perf.total_pnl
                        )
                    } else {
                        String::new() // 轻微盈亏,不添加提示
                    }
                }
            }
        } else {
            String::new() // 无历史数据,不添加提示
        };

        // 【多时间周期分析】超短线交易策略：5m微观、15m趋势、1h支撑阻力
        let (klines_5m_result, klines_15m_result, klines_1h_result) = tokio::join!(
            tokio::time::timeout(
                tokio::time::Duration::from_secs(10),
                self.exchange.get_klines(&symbol, "5m", Some(50))
            ),
            tokio::time::timeout(
                tokio::time::Duration::from_secs(10),
                self.exchange.get_klines(&symbol, "15m", Some(100))
            ),
            tokio::time::timeout(
                tokio::time::Duration::from_secs(10),
                self.exchange.get_klines(&symbol, "1h", Some(48))
            )
        );

        // 解析5m K线
        let klines_5m = match klines_5m_result {
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
                warn!("❌ 获取{}5mK线失败: {}", symbol, e);
                return Ok(());
            }
            Err(_) => {
                warn!("❌ 获取{}5mK线超时", symbol);
                return Ok(());
            }
        };

        // 解析15m K线
        let klines = match klines_15m_result {
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

        // 解析1h K线
        let klines_1h = match klines_1h_result {
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
                warn!("❌ 获取{}1hK线失败: {}", symbol, e);
                return Ok(());
            }
            Err(_) => {
                warn!("❌ 获取{}1hK线超时", symbol);
                return Ok(());
            }
        };

        if klines_1h.len() < 12 {
            warn!("⚠️  1h K线数据不足: {} (需要至少12根)", klines_1h.len());
            return Ok(());
        }

        if let Some(last_hour) = klines_1h.last() {
            info!(
                "🕒 1h 最新K线: 收盘价 ${:.4} | 成交量 {:.2}",
                last_hour.close, last_hour.volume
            );
        }

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
        let base_prompt =
            self.build_enhanced_prompt(&alert, &klines, &indicators, &key_levels, current_price);

        // 4.5 附加历史表现数据
        let prompt = format!("{}{}", base_prompt, history_prompt);

        info!("📝 发送给DeepSeek AI分析...");

        // 5. 调用DeepSeek API分析市场 - 添加超时保护
        let decision_result = tokio::time::timeout(
            tokio::time::Duration::from_secs(30),
            self.deepseek.analyze_market(&prompt),
        )
        .await;

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
        info!("   止损价: ${:.4}", decision.stop_loss.unwrap_or(0.0));
        info!("   止盈价: ${:.4}", decision.take_profit.unwrap_or(0.0));

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
        let current_position = self
            .exchange
            .get_positions()
            .await
            .ok()
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
                // 高信心: 最大仓位 2U + 最高杠杆 10x = 20U名义价值
                (self.max_position_usdt, self.max_leverage)
            }
            "MEDIUM" => {
                // 中信心: 中等仓位 1.5U + 中等杠杆 8x = 12U名义价值
                let mid_position = (self.min_position_usdt + self.max_position_usdt) / 2.0;
                let mid_leverage = (self.min_leverage + self.max_leverage) / 2;
                (mid_position, mid_leverage)
            }
            _ => {
                // 低信心: 最小仓位 1U + 最低杠杆 6x = 6U名义价值 (实际上LOW会被跳过)
                (self.min_position_usdt, self.min_leverage)
            }
        };

        let quantity = position_usdt * leverage as f64 / current_price;

        info!("💰 仓位配置:");
        info!(
            "   投入USDT: {:.2} (动态范围: {:.1}-{:.1}U)",
            position_usdt, self.min_position_usdt, self.max_position_usdt
        );
        info!(
            "   杠杆倍数: {}x (动态范围: {}-{}x)",
            leverage, self.min_leverage, self.max_leverage
        );
        info!("   开仓数量: {:.6} {}", quantity, alert.coin);
        info!(
            "   名义价值: {:.2} USDT ({}U × {}x)",
            position_usdt * leverage as f64,
            position_usdt,
            leverage
        );

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
                info!("   止损价: ${:.4}", decision.stop_loss.unwrap_or(0.0));
                info!("   止盈价: ${:.4}", decision.take_profit.unwrap_or(0.0));

                // 9. 自动设置止损止盈单
                info!("\n🎯 设置自动止损止盈单...");

                // 设置止损单
                let stop_loss_order_id = if let Some(sl_price) = decision.stop_loss {
                    match self
                        .exchange
                        .set_stop_loss(&symbol, side, quantity, sl_price)
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
                    }
                } else {
                    info!("   ⚠️  AI未提供止损价,跳过止损单设置");
                    None
                };

                // 设置止盈单
                let take_profit_order_id = if let Some(tp_price) = decision.take_profit {
                    match self
                        .exchange
                        .set_take_profit(&symbol, side, quantity, tp_price)
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
                    }
                } else {
                    info!("   📌 采用动态止盈策略(由AI监控持仓管理)");
                    None
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
        _klines: &[Kline],
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
            r#"你是一位顶尖的加密货币交易分析师,专精12小时内超短线操作,基于Valuescan主力资金监控系统执行交易。

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 交易标的: ${}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

【核心信号】Valuescan主力资金异动 (信号源优先级最高)
- 信号类型: {}
- 当前价格: ${:.6}
- 资金类型: {} (合约资金看主力,现货资金看大盘)
- 信号时间: {}

🔥 【ValueScan核心口诀】
1. "异动首次响,黄金千万两!" - 首次异动信号最重要
2. "alpha首次推,仓位闭眼堆!" - 首个Alpha信号高置信度
3. "fomo一现,热点出现" - FOMO信号代表市场焦点
4. 异动频繁→市场活跃可操作 | 异动冷清→多看少做
5. Alpha+FOMO组合 = 最强信号
6. 风险区+异动同时出现 → 不做

【辅助判断1】1h K线关键位 (主力建仓区域识别)
{}
动态位置: {}

【辅助判断2】15m技术指标 (入场时机确认)
- RSI(14): {:.2}
- MACD柱状: {:.4}
- 布林带位置: {}
- 均线状态: SMA5=${:.4} SMA20=${:.4}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🎯 【超短线决策原则】12小时内操作,快进快出
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ **HIGH信心开多条件** (系统自动配置: 2U × 10x = 20U名义价值):
- Valuescan首次资金流入异动
- 价格在1h支撑位上方 OR 刚突破支撑位
- 5m/15m趋势向上,无顶部反转形态
- RSI < 75 (非严重超买)
- 关键: 主力资金持续流入,异动频繁

✅ **MEDIUM信心开多条件** (系统自动配置: 1.5U × 8x = 12U名义价值):
- 资金流入信号但非首次
- 价格在支撑与阻力之间
- 技术指标中性偏多
- RSI 50-70区间

❌ **LOW信心/SKIP条件** (系统自动配置: 1U × 6x = 6U,但实际会跳过交易):
- 异动信号冷清,市场不活跃
- 价格接近阻力位但未突破
- RSI > 80 严重超买
- 5m/15m出现明显顶部形态
- 关键位不明确

🔻 **做空条件** (仅限以下情况):
- Valuescan主力资金撤离/出逃信号
- 价格跌破1h主力支撑位
- 5m出现明显顶部反转
- RSI > 25 (避免抄底被套)

⏱️ **超短线风控**:
- 目标: 12小时内操作
- 止损: 入场价-2% OR 最近支撑位-2% (取近的)
- 止盈: 动态管理(AI监控),不设固定目标
- 时间止损: 4小时未盈利>1%强制离场

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📋 【输出格式】严格JSON
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

{{
    "signal": "BUY|SELL|HOLD|SKIP",
    "confidence": "HIGH|MEDIUM|LOW",
    "stop_loss": 止损价格(数字),
    "take_profit": 止盈价格(数字) 或 null (动态管理),
    "reason": "决策理由(限100字,必须包含:信号类型+关键位状态+趋势判断)"
}}

**confidence解释**:
- HIGH: 首次异动+关键位有利+趋势强 → 系统自动: 2U×10x
- MEDIUM: 非首次信号或技术指标中性 → 系统自动: 1.5U×8x  
- LOW: 信号弱或风险高 → 系统自动跳过交易

**signal决策核心**:
1. 频道信号占权重70% (主力资金最重要)
2. 1h关键位占权重20% (支撑/阻力判断)
3. 技术指标占权重10% (仅确认入场时机)

现在请分析以上数据,给出明确的12小时超短线交易决策！
"#,
            alert.coin,
            alert_type_desc,
            current_price,
            alert.fund_type,
            alert.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            self.format_key_levels(
                key_levels,
                current_price,
                &nearest_support,
                &nearest_resistance
            ),
            self.format_entry_condition(&nearest_support, &nearest_resistance, current_price),
            indicators.rsi,
            indicators.macd - indicators.macd_signal,
            self.get_bb_position(current_price, indicators),
            indicators.sma_5,
            indicators.sma_20,
        )
    }

    fn get_bb_position(&self, price: f64, indicators: &TechnicalIndicators) -> &str {
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
    info!("🚀 集成AI交易系统 - Alpha/FOMO交易版");
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
                info!(
                    "✅ 目标频道已解析: {} (ID: {})",
                    channel.title(),
                    channel.id()
                );
                target_channel_id = Some(channel.id());
            }
        }
    }

    info!("✅ 已缓存 {} 个频道实体 (防止消息丢失)", cached_channels);

    let target_channel_id = match target_channel_id {
        Some(id) => id,
        None => {
            anyhow::bail!(
                "❌ 无法找到目标频道 (ID: {}),请确保已加入该频道",
                trader.fund_channel_id
            );
        }
    };

    info!("📡 开始实时监控...");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // 实时监控循环
    loop {
        match trader.telegram_client.next_update().await {
            Ok(Update::NewMessage(message)) if !message.outgoing() => match message.chat() {
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
            },
            Err(e) => {
                error!("❌ Telegram连接错误: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
            _ => {}
        }
    }
}
