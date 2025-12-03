/// 集成AI交易系统 - 整合主力资金监控 + DeepSeek AI + 多交易所执行
///
/// 功能：
/// 1. 监控Telegram主力资金频道(Valuescan 2254462672)
/// 2. 筛选Alpha/FOMO高潜力币种
/// 3. 获取技术数据（K线、指标、关键位）
/// 4. DeepSeek AI综合分析决策
/// 5. 多交易所并发执行
/// 6. 严格风控管理
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::env;
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant};
use teloxide::{prelude::Requester, Bot};
use tokio::{
    io::AsyncWriteExt,
    sync::{Mutex, RwLock},
};

pub const POSITION_CHECK_INTERVAL_SECS: u64 = 180; // P1优化: 从600s(10分钟)减少到180s(3分钟),提升风控响应速度
#[allow(dead_code)] // 后续用于切换增强版持仓分析逻辑
const USE_ENHANCED_ANALYSIS: bool = false;
lazy_static! {
    static ref USE_VALUESCAN_V2: bool = env::var("USE_VALUESCAN_V2")
        .map(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            matches!(normalized.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(false);
}
#[allow(dead_code)]
const VOLATILITY_CACHE_TTL_SECS: u64 = 60;
#[allow(dead_code)]
const VOLATILITY_TIMEOUT_SECS: u64 = 5;
#[allow(dead_code)]
const VOLATILITY_LOOKBACK: usize = 20;
#[allow(dead_code)]
const DEFAULT_VOLATILITY_PERCENT: f64 = 2.0;
#[allow(dead_code)]
const MEME_COINS: [&str; 7] = [
    "PUMPUSDT",
    "GIGGLEUSDT",
    "POPCATUSDT",
    "WIFUSDT",
    "SHIBUSDT",
    "DOGEUSDT",
    "PEPEUSDT",
];

use rust_trading_bot::database::{AiAnalysisRecord, Database, TradeRecord as DbTradeRecord};
use rust_trading_bot::support_analyzer::{Kline as SupportKline, SupportAnalyzer};
use rust_trading_bot::{
    binance_client::BinanceClient,
    deepseek_client::{
        ActionParams, DeepSeekClient, EnhancedPositionAnalysis, Kline, PositionManagementDecision,
        TechnicalIndicators, TradingSignal,
    },
    entry_zone_analyzer::{EntryAction, EntryDecision, EntryZone, EntryZoneAnalyzer},
    exchange_trait::{ExchangeClient, Position},
    gemini_client::GeminiClient,
    key_level_finder::KeyLevelFinder,
    launch_signal_detector::LaunchSignalDetector,
    prompt_templates::{
        build_entry_prompt_v1, build_entry_prompt_v2, build_position_prompt_v2, EntryPromptContext,
        PositionPromptContext,
    },
    signals::{AlertType, FundAlert, MessageParser, SignalContext},
    staged_position_manager::{StagedPosition, StagedPositionManager},
    technical_analysis::TechnicalAnalyzer,
    trading::OrderManager,
    valuescan_v2::TradingSignalV2,
};

/// 延迟开仓队列记录 - 首次未开仓的币种,等待更好时机
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingEntry {
    symbol: String,
    first_signal_time: DateTime<Utc>,
    last_analysis_time: DateTime<Utc>,
    alert: FundAlert,
    reject_reason: String, // 为什么首次被拒绝: "价格不符"/"AI SKIP"/"等待回调"
    retry_count: u32,      // 已重试次数
    fund_escape_detected_at: Option<DateTime<Utc>>, // 首次检测到资金出逃的时间
}

/// 持仓追踪器快照 - 用于无锁读取
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerSnapshot {
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

/// 持倉追蹤資訊
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionTracker {
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

/// 缓存批量AI评估所需的行情上下文，避免重复获取K线
pub struct PositionMarketContext {
    klines_5m: Vec<Kline>,
    klines_15m: Vec<Kline>,
    klines_1h: Vec<Kline>,
    indicators: TechnicalIndicators,
}

/// 保存批量AI评估完成后执行交易动作所需的持仓信息
pub struct BatchActionContext {
    side: String,
    entry_price: f64,
    quantity: f64,
    stop_loss_order_id: Option<String>,
    take_profit_order_id: Option<String>,
}

/// 统一封装AI评估所需的完整上下文，复用单次与批量流程
struct PreparedPositionContext {
    symbol: String,
    side: String,
    entry_price: f64,
    stop_loss_price: f64,
    current_price: f64,
    quantity: f64,
    duration: f64,
    profit_pct: f64,
    stop_loss_order_id: Option<String>,
    take_profit_order_id: Option<String>,
    market: PositionMarketContext,
    support_text: String,
    deviation_desc: String,
    current_stop_loss: Option<f64>,
    current_take_profit: Option<f64>,
}

impl PreparedPositionContext {
    fn to_prompt_context(&self) -> PositionPromptContext<'_> {
        PositionPromptContext {
            symbol: &self.symbol,
            side: &self.side,
            entry_price: self.entry_price,
            current_price: self.current_price,
            profit_pct: self.profit_pct,
            hold_duration_hours: self.duration,
            klines_5m: &self.market.klines_5m,
            klines_15m: &self.market.klines_15m,
            klines_1h: &self.market.klines_1h,
            indicators: &self.market.indicators,
            support_text: &self.support_text,
            deviation_desc: &self.deviation_desc,
            current_stop_loss: self.current_stop_loss,
            current_take_profit: self.current_take_profit,
            funding_rate_info: None,
        }
    }

    fn to_batch_input(
        &self,
    ) -> (
        String,
        String,
        f64,
        f64,
        f64,
        f64,
        Vec<Kline>,
        Vec<Kline>,
        Vec<Kline>,
        TechnicalIndicators,
    ) {
        (
            self.symbol.clone(),
            self.side.clone(),
            self.entry_price,
            self.current_price,
            self.profit_pct,
            self.duration,
            self.market.klines_5m.clone(),
            self.market.klines_15m.clone(),
            self.market.klines_1h.clone(),
            self.market.indicators.clone(),
        )
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct VolatilityCacheEntry {
    value: f64,
    cached_at: Instant,
}

/// 触发单跟踪记录
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct TriggerOrderRecord {
    order_id: String,
    symbol: String,
    position_side: String,
    trigger_price: f64,
    action: String, // "OPEN" or "CLOSE"
    created_at: DateTime<Utc>,
    reason: String,
}

/// 持仓监控阶段需要执行的动作，采用“先收集再处理”策略避免锁重入
#[derive(Debug)]
pub enum PositionAction {
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
        stop_loss_price: f64, // ✅ Bug Fix: 保存原始止损价格,部分平仓后重设止损单使用
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

/// 描述AI分析前的准备结果
enum PositionEvaluationStep {
    Skip,
    Immediate(PositionAction),
    Context(PreparedPositionContext),
}

/// 对追踪器的更新操作，统一在短暂写锁中落盘
#[derive(Debug)]
pub enum TrackerMutation {
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
pub struct SignalRecord {
    timestamp: String,
    signal: String,
    confidence: String,
    reason: String,
    price: f64,
}

/// 交易信號歷史
pub struct SignalHistory {
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

    #[allow(dead_code)] // 保留供未来频率分析使用
    fn get_recent(&self, count: usize) -> Vec<&SignalRecord> {
        self.signals.iter().rev().take(count).collect()
    }

    #[allow(dead_code)] // 保留供未来频率分析使用
    fn count_signal(&self, signal: &str, last_n: usize) -> usize {
        self.signals
            .iter()
            .rev()
            .take(last_n)
            .filter(|s| s.signal == signal)
            .count()
    }
}

pub struct IntegratedAITrader {
    pub exchange: Arc<BinanceClient>,
    pub deepseek: Arc<DeepSeekClient>,
    pub gemini: Arc<GeminiClient>,
    pub analyzer: Arc<TechnicalAnalyzer>,
    #[allow(dead_code)] // 保留供未来多策略扩展使用
    pub level_finder: Arc<KeyLevelFinder>,

    // 新策略模块
    pub entry_zone_analyzer: Arc<EntryZoneAnalyzer>,
    pub launch_detector: Arc<LaunchSignalDetector>,
    pub staged_manager: Arc<RwLock<StagedPositionManager>>,

    #[allow(dead_code)] // 保留供未来Alpha/FOMO分类使用
    pub alpha_keywords: Vec<String>,
    #[allow(dead_code)] // 保留供未来Alpha/FOMO分类使用
    pub fomo_keywords: Vec<String>,

    // 交易配置 - 动态范围
    pub min_position_usdt: f64, // 最小仓位 1 USDT
    pub max_position_usdt: f64, // 最大仓位 2 USDT
    pub min_leverage: u32,      // 最小杠杆 6x
    pub max_leverage: u32,      // 最大杠杆 10x

    // 内存管理配置
    pub max_tracked_coins: usize, // tracked_coins 最大数量
    pub coin_ttl_hours: i64,      // 币种追踪过期时间(小时)

    // 状态跟踪
    pub tracked_coins: Arc<RwLock<HashMap<String, FundAlert>>>,
    pub position_trackers: Arc<RwLock<HashMap<String, PositionTracker>>>,
    pub signal_history: Arc<RwLock<SignalHistory>>,
    pub last_analysis_time: Arc<RwLock<HashMap<String, DateTime<Utc>>>>, // 【优化1】信号去重
    #[allow(dead_code)]
    pub volatility_cache: Arc<RwLock<HashMap<String, VolatilityCacheEntry>>>,
    pub active_trigger_orders: Arc<Mutex<Vec<TriggerOrderRecord>>>,
    pub pending_entries: Arc<RwLock<HashMap<String, PendingEntry>>>, // 延迟开仓队列
    pub db: Database,                                                // 直接写入数据库
    pub order_manager: OrderManager,
    pub telegram_bot: Option<Arc<Bot>>,
}

impl IntegratedAITrader {
    pub async fn new(
        exchange: BinanceClient,
        deepseek_api_key: String,
        gemini_api_key: String,
        db: Database,
    ) -> Self {
        let exchange = Arc::new(exchange);
        let telegram_bot = match env::var("TELEGRAM_BOT_TOKEN") {
            Ok(token) if !token.trim().is_empty() => {
                info!("💬 Telegram 告警已启用");
                Some(Arc::new(Bot::new(token)))
            }
            Ok(_) => {
                warn!("⚠️ TELEGRAM_BOT_TOKEN 为空, Telegram 告警不可用");
                None
            }
            Err(_) => {
                warn!("⚠️ 未配置 TELEGRAM_BOT_TOKEN, Telegram 告警不可用");
                None
            }
        };
        Self {
            order_manager: OrderManager::new(exchange.clone()),
            exchange,
            deepseek: Arc::new(DeepSeekClient::new(deepseek_api_key)),
            gemini: Arc::new(GeminiClient::new(gemini_api_key)),
            analyzer: Arc::new(TechnicalAnalyzer::new()),
            level_finder: Arc::new(KeyLevelFinder::new()),

            // 初始化新策略模块
            entry_zone_analyzer: Arc::new(EntryZoneAnalyzer::default()),
            launch_detector: Arc::new(LaunchSignalDetector::default()),
            staged_manager: Arc::new(RwLock::new(StagedPositionManager::default())),

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

            min_position_usdt: 5.0, // 单笔固定 5 USDT (满足Binance最小订单要求)
            max_position_usdt: 5.0,
            min_leverage: 5,  // 修改为5-15x杠杆范围: Low信心=5x
            max_leverage: 15, // High信心=15x, Medium信心=10x

            // 内存管理配置
            max_tracked_coins: 100, // 最多追踪 100 个币种
            coin_ttl_hours: 24,     // 24 小时后自动过期

            tracked_coins: Arc::new(RwLock::new(HashMap::new())),
            position_trackers: Arc::new(RwLock::new(HashMap::new())),
            signal_history: Arc::new(RwLock::new(SignalHistory::new(30))),
            last_analysis_time: Arc::new(RwLock::new(HashMap::new())), // 【优化1】初始化去重map
            volatility_cache: Arc::new(RwLock::new(HashMap::new())),
            active_trigger_orders: Arc::new(Mutex::new(Vec::new())),
            pending_entries: Arc::new(RwLock::new(HashMap::new())),
            db,
            telegram_bot,
        }
    }

    /// 解析资金异动消息
    /// 判断是否为Alpha/FOMO机会
    #[allow(dead_code)] // 保留供未来Alpha/FOMO分类使用
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

    /// 判断是否属于 MEME 币种，触发更严格风控
    fn is_meme_coin(symbol: &str) -> bool {
        MEME_COINS
            .iter()
            .any(|meme| meme.eq_ignore_ascii_case(symbol))
    }

    /// 清理过期的追踪币种 - 防止内存泄漏
    async fn cleanup_tracked_coins(&self) {
        MessageParser::cleanup_tracked_coins(self).await;
    }

    /// 监控并调整触发单
    async fn monitor_trigger_orders(&self) -> Result<()> {
        let snapshot = {
            let orders = self.active_trigger_orders.lock().await;
            if orders.is_empty() {
                return Ok(());
            }
            orders.clone()
        };

        let mut orders_to_remove: HashSet<String> = HashSet::new();

        for record in snapshot {
            match self
                .exchange
                .get_order_status_detail(&record.symbol, &record.order_id)
                .await
            {
                Ok(status) => {
                    let status_text = status.status.as_str();
                    if matches!(status_text, "FILLED" | "CANCELED" | "EXPIRED") {
                        info!("🔔 触发单 {} 已完成: {}", record.order_id, status.status);
                        orders_to_remove.insert(record.order_id.clone());
                        continue;
                    }
                }
                Err(e) => {
                    warn!("⚠️ 查询触发单失败: {} - {}", record.order_id, e);
                    continue;
                }
            }

            let current_price = match self.exchange.get_current_price(&record.symbol).await {
                Ok(price) => price,
                Err(e) => {
                    warn!(
                        "⚠️ 获取 {} 当前价格失败, 暂不调整触发单 {}: {}",
                        record.symbol, record.order_id, e
                    );
                    continue;
                }
            };

            let should_cancel = self
                .should_cancel_trigger_order(&record, current_price)
                .await;

            if should_cancel {
                info!(
                    "🗑️ 取消不再合理的触发单: {} @ {:.4}",
                    record.symbol, record.trigger_price
                );
                if let Err(e) = self
                    .order_manager
                    .cancel_order(&record.symbol, &record.order_id)
                    .await
                {
                    warn!("⚠️ 取消触发单失败: {}", e);
                } else {
                    orders_to_remove.insert(record.order_id.clone());
                }
            }
        }

        if !orders_to_remove.is_empty() {
            let mut orders = self.active_trigger_orders.lock().await;
            orders.retain(|record| !orders_to_remove.contains(&record.order_id));
        }

        Ok(())
    }

    /// 判断触发单是否应该取消
    async fn should_cancel_trigger_order(
        &self,
        record: &TriggerOrderRecord,
        current_price: f64,
    ) -> bool {
        let age = Utc::now() - record.created_at;
        if age.num_hours() > 4 {
            info!(
                "⏰ 触发单 {} 已挂单 {}h,自动取消",
                record.order_id,
                age.num_hours()
            );
            return true;
        }

        let trigger_price = if record.trigger_price.abs() < f64::EPSILON {
            f64::EPSILON
        } else {
            record.trigger_price
        };
        let price_deviation = ((current_price - trigger_price).abs() / trigger_price) * 100.0;

        if record.action.eq_ignore_ascii_case("OPEN") && price_deviation > 5.0 {
            info!(
                "📉 触发价 {:.4} 与当前价 {:.4} 偏离 {:.1}%,取消开仓触发单",
                record.trigger_price, current_price, price_deviation
            );
            return true;
        }

        false
    }

    /// 处理新消息 - 所有信号(包括出逃)都送给AI判断
    #[allow(dead_code)]
    async fn handle_message(&self, text: &str) -> Result<()> {
        MessageParser::handle_message(self, text).await
    }

    /// 处理来自 Web API 的 Valuescan 信号
    pub async fn handle_valuescan_message(
        &self,
        symbol: &str,
        message_text: &str,
        score: i32,
        signal_type: &str,
    ) -> Result<()> {
        MessageParser::handle_valuescan_message(self, symbol, message_text, score, signal_type)
            .await
    }

    #[allow(dead_code)]
    async fn handle_incoming_alert(
        &self,
        alert: FundAlert,
        raw_message: &str,
        persist_signal: bool,
    ) -> Result<()> {
        MessageParser::handle_incoming_alert(self, alert, raw_message, persist_signal).await
    }

    #[allow(dead_code)]
    async fn process_classified_alert(&self, alert: FundAlert) -> Result<()> {
        MessageParser::process_classified_alert(self, alert).await
    }

    /// 检查是否应该因频繁交易而跳过执行
    #[allow(dead_code)] // 保留供未来频率过滤使用
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

    /// 统一抓取多周期K线并计算技术指标，供批量/单次AI评估复用
    async fn collect_position_market_context(
        &self,
        symbol: &str,
    ) -> Result<Option<PositionMarketContext>> {
        fn convert_exchange_klines(raw: Vec<Vec<f64>>) -> Vec<Kline> {
            raw.into_iter()
                .map(|candle| Kline {
                    timestamp: candle.get(0).copied().unwrap_or_default() as i64,
                    open: candle.get(1).copied().unwrap_or_default(),
                    high: candle.get(2).copied().unwrap_or_default(),
                    low: candle.get(3).copied().unwrap_or_default(),
                    close: candle.get(4).copied().unwrap_or_default(),
                    volume: candle.get(5).copied().unwrap_or_default(),
                    quote_volume: candle.get(6).copied().unwrap_or(0.0),
                    taker_buy_volume: candle.get(7).copied().unwrap_or(0.0),
                    taker_buy_quote_volume: candle.get(8).copied().unwrap_or(0.0),
                })
                .collect()
        }

        let (klines_5m_result, klines_15m_result, klines_1h_result) = tokio::join!(
            tokio::time::timeout(
                tokio::time::Duration::from_secs(10),
                self.exchange.get_klines(symbol, "5m", Some(50)),
            ),
            tokio::time::timeout(
                tokio::time::Duration::from_secs(10),
                self.exchange.get_klines(symbol, "15m", Some(100)),
            ),
            tokio::time::timeout(
                tokio::time::Duration::from_secs(10),
                self.exchange.get_klines(symbol, "1h", Some(48)),
            ),
        );

        let klines_5m = match klines_5m_result {
            Ok(Ok(data)) => convert_exchange_klines(data),
            Ok(Err(e)) => {
                warn!("⚠️  获取{}5mK线失败: {}, 跳过AI评估", symbol, e);
                return Ok(None);
            }
            Err(_) => {
                warn!("⚠️  获取{}5mK线超时, 跳过AI评估", symbol);
                return Ok(None);
            }
        };

        let klines_15m = match klines_15m_result {
            Ok(Ok(data)) => convert_exchange_klines(data),
            Ok(Err(e)) => {
                warn!("⚠️  获取{}15mK线失败: {}, 跳过AI评估", symbol, e);
                return Ok(None);
            }
            Err(_) => {
                warn!("⚠️  获取{}15mK线超时, 跳过AI评估", symbol);
                return Ok(None);
            }
        };

        let klines_1h = match klines_1h_result {
            Ok(Ok(data)) => convert_exchange_klines(data),
            Ok(Err(e)) => {
                warn!("⚠️  获取{}1hK线失败: {}, 跳过AI评估", symbol, e);
                return Ok(None);
            }
            Err(_) => {
                warn!("⚠️  获取{}1hK线超时, 跳过AI评估", symbol);
                return Ok(None);
            }
        };

        if klines_15m.len() < 20 {
            warn!(
                "⚠️  K线数据不足: {} (需要至少20根), 跳过AI评估",
                klines_15m.len()
            );
            return Ok(None);
        }

        let indicators = self.analyzer.calculate_indicators(&klines_15m);

        Ok(Some(PositionMarketContext {
            klines_5m,
            klines_15m,
            klines_1h,
            indicators,
        }))
    }

    #[allow(dead_code)]
    async fn store_volatility_cache(&self, symbol: &str, value: f64) {
        let mut cache = self.volatility_cache.write().await;
        cache.insert(
            symbol.to_string(),
            VolatilityCacheEntry {
                value,
                cached_at: Instant::now(),
            },
        );
    }

    /// 计算市场波动率 (基于ATR或近期价格标准差)
    /// 返回波动率百分比 (0-100)
    #[allow(dead_code)]
    async fn calculate_volatility(&self, symbol: &str) -> Result<f64> {
        if let Some(entry) = {
            let cache = self.volatility_cache.read().await;
            cache.get(symbol).copied()
        } {
            if entry.cached_at.elapsed() < StdDuration::from_secs(VOLATILITY_CACHE_TTL_SECS) {
                debug!("📊 波动率缓存命中: {} => {:.2}%", symbol, entry.value);
                return Ok(entry.value);
            }
        }

        let klines_result = tokio::time::timeout(
            tokio::time::Duration::from_secs(VOLATILITY_TIMEOUT_SECS),
            self.exchange
                .get_klines(symbol, "15m", Some(VOLATILITY_LOOKBACK)),
        )
        .await;

        let raw_klines = match klines_result {
            Ok(Ok(data)) => data,
            Ok(Err(err)) => {
                warn!(
                    "⚠️  获取{} 15m K线计算波动率失败: {}，使用默认值",
                    symbol, err
                );
                self.store_volatility_cache(symbol, DEFAULT_VOLATILITY_PERCENT)
                    .await;
                return Ok(DEFAULT_VOLATILITY_PERCENT);
            }
            Err(_) => {
                warn!(
                    "⚠️  获取{} 15m K线计算波动率超时(>{}s)，使用默认值",
                    symbol, VOLATILITY_TIMEOUT_SECS
                );
                self.store_volatility_cache(symbol, DEFAULT_VOLATILITY_PERCENT)
                    .await;
                return Ok(DEFAULT_VOLATILITY_PERCENT);
            }
        };

        let klines: Vec<Kline> = raw_klines
            .into_iter()
            .map(|candle| Kline {
                timestamp: candle.get(0).copied().unwrap_or_default() as i64,
                open: candle.get(1).copied().unwrap_or_default(),
                high: candle.get(2).copied().unwrap_or_default(),
                low: candle.get(3).copied().unwrap_or_default(),
                close: candle.get(4).copied().unwrap_or_default(),
                volume: candle.get(5).copied().unwrap_or_default(),
                quote_volume: candle.get(6).copied().unwrap_or(0.0),
                taker_buy_volume: candle.get(7).copied().unwrap_or(0.0),
                taker_buy_quote_volume: candle.get(8).copied().unwrap_or(0.0),
            })
            .collect();

        if klines.len() < 2 {
            warn!(
                "⚠️  {} 15m K线数量不足({})，无法计算波动率，使用默认值",
                symbol,
                klines.len()
            );
            self.store_volatility_cache(symbol, DEFAULT_VOLATILITY_PERCENT)
                .await;
            return Ok(DEFAULT_VOLATILITY_PERCENT);
        }

        let mut prev_close = klines[0].close;
        let mut tr_total = 0.0;
        let mut samples = 0usize;

        for candle in klines.iter().skip(1) {
            let hl = (candle.high - candle.low).abs();
            let hc = (candle.high - prev_close).abs();
            let lc = (candle.low - prev_close).abs();
            let tr = hl.max(hc).max(lc);
            tr_total += tr;
            samples += 1;
            prev_close = candle.close;
        }

        if samples == 0 {
            warn!("⚠️  {} 触发波动率计算时 TR 样本为空，使用默认值", symbol);
            self.store_volatility_cache(symbol, DEFAULT_VOLATILITY_PERCENT)
                .await;
            return Ok(DEFAULT_VOLATILITY_PERCENT);
        }

        let atr = tr_total / samples as f64;
        let current_price = klines
            .last()
            .map(|c| c.close)
            .filter(|price| *price > f64::EPSILON)
            .unwrap_or(0.0);

        if current_price <= f64::EPSILON {
            warn!(
                "⚠️  {} 当前价格异常({:.6})，无法计算波动率，使用默认值",
                symbol, current_price
            );
            self.store_volatility_cache(symbol, DEFAULT_VOLATILITY_PERCENT)
                .await;
            return Ok(DEFAULT_VOLATILITY_PERCENT);
        }

        let volatility = ((atr / current_price) * 100.0).max(0.0);
        debug!(
            "📊 {} 波动率计算完成: ATR {:.4}, Price {:.4}, Vol {:.2}%",
            symbol, atr, current_price, volatility
        );

        self.store_volatility_cache(symbol, volatility).await;
        Ok(volatility)
    }

    /// 【P0-2】验证当前价格是否仍处于有效入场区，避免信号延迟导致追高
    async fn validate_entry_zone(
        &self,
        signal_price: f64,
        current_price: f64,
        entry_zone: (f64, f64),
        indicators: &TechnicalIndicators,
        is_ai_override: bool,
    ) -> Result<bool> {
        // 1. 信号延迟检查：当前价相对信号价偏离超过 2% 则拒绝，处理信号价为 0 的异常
        if signal_price > 0.0 {
            let deviation = (current_price - signal_price).abs() / signal_price;
            if deviation > 0.02 {
                warn!("❌ 信号延迟过大: 偏离{:.2}%, 拒绝入场", deviation * 100.0);
                return Ok(false);
            }
        } else {
            warn!(
                "⚠️ signal_price为0,跳过偏离度检查 (当前价: ${:.4})",
                current_price
            );
        }

        // 2. 入场区边界检查 - 动态容差
        let (entry_zone_min, entry_zone_max) = entry_zone;
        let price_tolerance = if is_ai_override {
            // AI覆盖：根据 RSI 与区间波动幅度动态扩展容差
            let rsi = indicators.rsi;
            let price_range = (entry_zone_max - entry_zone_min) / entry_zone_min * 100.0;

            if rsi > 65.0 || price_range > 5.0 {
                0.25
            } else if rsi > 45.0 {
                0.20
            } else {
                0.15
            }
        } else {
            0.03
        };
        let extended_min = entry_zone_min * (1.0 - price_tolerance);
        let extended_max = entry_zone_max * (1.0 + price_tolerance);

        if current_price < extended_min || current_price > extended_max {
            warn!(
                "❌ 价格不在入场区 [{:.4}, {:.4}] (扩展), 当前{:.4}, 拒绝入场",
                extended_min, extended_max, current_price
            );
            return Ok(false);
        }

        if is_ai_override && (current_price < entry_zone_min || current_price > entry_zone_max) {
            info!(
                "⚠️  价格超出标准入场区,但在AI动态容差范围内 ({:.1}%, RSI={:.1})",
                price_tolerance * 100.0,
                indicators.rsi
            );
            info!(
                "   标准区间: [{:.4}, {:.4}]",
                entry_zone_min, entry_zone_max
            );
            info!("   扩展区间: [{:.4}, {:.4}]", extended_min, extended_max);
            info!("   当前价格: {:.4}", current_price);
        }

        // 3. RSI 超买检查
        if indicators.rsi > 75.0 {
            warn!("❌ RSI严重超买 {:.1}, 拒绝入场", indicators.rsi);
            return Ok(false);
        }

        Ok(true)
    }

    /// 将 AI 决策转换为 PositionAction，统一处理日志与边界情况
    fn build_action_from_decision(
        symbol: &str,
        side: &str,
        entry_price: f64,
        stop_loss_price: f64, // ✅ Bug Fix: 止损价格参数
        quantity: f64,
        stop_loss_order_id: Option<String>,
        take_profit_order_id: Option<String>,
        decision: &PositionManagementDecision,
        min_notional: f64,
        current_price: f64,
    ) -> Option<PositionAction> {
        match decision.action.as_str() {
            "HOLD" => {
                info!("✅ AI 建议继续持有 {}", symbol);
                None
            }
            "PARTIAL_CLOSE" => {
                // ✅ 使用默认50%当AI未提供百分比时
                let close_pct = decision.close_percentage.unwrap_or(50.0);
                if decision.close_percentage.is_none() {
                    warn!("⚠️  AI 建议部分平仓但未提供百分比,使用默认50%");
                }
                info!("📉 AI 建议部分平仓 {} ({}%)", symbol, close_pct);
                let close_quantity = (quantity * (close_pct / 100.0)).clamp(0.0, quantity);
                let remaining_quantity = (quantity - close_quantity).max(0.0);

                if close_quantity <= f64::EPSILON {
                    warn!("⚠️  计算得到的平仓数量过小, 跳过本次部分平仓");
                    None
                } else {
                    // ✅ 智能部分平仓比率调整: 动态MIN_NOTIONAL + 使用当前价格
                    let position_total_value = quantity * current_price;
                    let suggested_close_value = close_quantity * current_price;

                    if suggested_close_value < min_notional {
                        // 计算满足MIN_NOTIONAL的最小平仓比率
                        let min_ratio_pct = (min_notional / position_total_value * 100.0).ceil();

                        if min_ratio_pct <= 100.0 {
                            // 调整到最小比率
                            let adjusted_close_pct = min_ratio_pct;
                            let adjusted_close_qty = quantity * (adjusted_close_pct / 100.0);
                            let adjusted_close_value = adjusted_close_qty * current_price;

                            warn!(
                                "⚠️ {} 部分平仓比率调整: AI建议{:.0}% (${:.2}) → 实际执行{:.0}% (${:.2})，满足MIN_NOTIONAL ${:.0}",
                                symbol, close_pct, suggested_close_value, adjusted_close_pct, adjusted_close_value, min_notional
                            );

                            let adjusted_remaining = (quantity - adjusted_close_qty).max(0.0);
                            Some(PositionAction::PartialClose {
                                symbol: symbol.to_string(),
                                side: side.to_string(),
                                close_quantity: adjusted_close_qty,
                                close_pct: adjusted_close_pct,
                                entry_price,
                                stop_loss_price,
                                remaining_quantity: adjusted_remaining,
                                stop_loss_order_id,
                            })
                        } else {
                            // 持仓总价值小于MIN_NOTIONAL,转为全部平仓
                            warn!(
                                "⚠️ {} 持仓总价值(${:.2}) < MIN_NOTIONAL(${:.0})，无法部分平仓，执行全部平仓",
                                symbol, position_total_value, min_notional
                            );
                            Some(PositionAction::FullClose {
                                symbol: symbol.to_string(),
                                side: side.to_string(),
                                quantity,
                                reason: "min_notional_full_close".to_string(),
                            })
                        }
                    } else {
                        Some(PositionAction::PartialClose {
                            symbol: symbol.to_string(),
                            side: side.to_string(),
                            close_quantity,
                            close_pct,
                            entry_price,
                            stop_loss_price,
                            remaining_quantity,
                            stop_loss_order_id,
                        })
                    }
                }
            }
            "FULL_CLOSE" => {
                info!("🚨 AI 建议全部平仓 {}", symbol);
                Some(PositionAction::FullClose {
                    symbol: symbol.to_string(),
                    side: side.to_string(),
                    quantity,
                    reason: "ai_decision".to_string(),
                })
            }
            "SET_LIMIT_ORDER" => {
                if let Some(limit_price) = decision.limit_price {
                    info!("🎯 AI 建议设置限价止盈单 {} @ ${:.4}", symbol, limit_price);
                    Some(PositionAction::SetLimitOrder {
                        symbol: symbol.to_string(),
                        side: side.to_string(),
                        quantity,
                        limit_price,
                        take_profit_order_id,
                    })
                } else {
                    warn!("⚠️  AI 建议设置限价单但未提供价格,保持持仓");
                    None
                }
            }
            other => {
                warn!("⚠️  未知的 AI 决策动作: {}, 保持持仓", other);
                None
            }
        }
    }

    /// 持仓监控线程 - 4小时超时止损 + 分级止盈 + 内存管理
    pub async fn monitor_positions(self: Arc<Self>) {
        info!("🔍 持仓监控线程已启动");

        let mut cleanup_counter = 0;
        let mut trigger_monitor_counter = 0;
        let mut orphaned_order_cleanup_counter = 0;
        let mut tracker_sync_counter = 0;

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(
                POSITION_CHECK_INTERVAL_SECS,
            ))
            .await; // 由于已设置止盈止损单,AI评估频率可降低至3-5分钟

            cleanup_counter += 1;
            trigger_monitor_counter += 1;
            orphaned_order_cleanup_counter += 1;
            tracker_sync_counter += 1;

            if trigger_monitor_counter >= 2 {
                if let Err(e) = self.monitor_trigger_orders().await {
                    warn!("⚠️ 触发单监控失败: {}", e);
                }
                trigger_monitor_counter = 0;
            }

            // 每 12 次检查(60分钟)执行一次全局清理
            if cleanup_counter >= 12 {
                info!("⏰ 开始执行定期内存清理...");
                self.cleanup_tracked_coins().await;
                self.cleanup_orphaned_trackers().await;
                cleanup_counter = 0;
                info!("✅ 定期内存清理完成");
            }

            // 每 10 次检查(30分钟)执行一次孤立触发单清理
            if orphaned_order_cleanup_counter >= 10 {
                if let Err(e) = self.cleanup_orphaned_trigger_orders().await {
                    warn!("⚠️ 孤立触发单清理失败: {}", e);
                }
                orphaned_order_cleanup_counter = 0;
            }

            if tracker_sync_counter >= 3 {
                if let Err(e) = self.sync_position_trackers().await {
                    warn!("⚠️ Tracker 同步失败: {}", e);
                }
                tracker_sync_counter = 0;
            }

            // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
            // 【每次循环】检查止盈止损互斥: 一方成交则取消另一方
            // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
            if let Err(e) = self.check_sl_tp_mutual_exclusion().await {
                warn!("⚠️ 止盈止损互斥检查失败: {}", e);
            }

            #[derive(Clone)]
            #[allow(dead_code)] // leverage字段保留供未来使用
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

            // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
            // 【新增】第一步: 检查试探持仓,检测启动信号并执行补仓
            // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
            {
                let staged_manager = self.staged_manager.read().await;
                let trial_positions: Vec<String> = staged_manager
                    .positions
                    .iter()
                    .filter_map(|(symbol, pos)| {
                        if matches!(
                            pos.stage,
                            rust_trading_bot::staged_position_manager::PositionStage::TrialPosition
                        ) {
                            Some(symbol.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                drop(staged_manager);

                for symbol in trial_positions {
                    info!("\n🔍 检查试探持仓: {}", symbol);

                    // 获取多周期K线数据 (1m, 5m, 15m, 1h)
                    let (klines_1m_result, klines_5m_result, klines_15m_result, klines_1h_result) = tokio::join!(
                        tokio::time::timeout(
                            tokio::time::Duration::from_secs(10),
                            self.exchange.get_klines(&symbol, "1m", Some(10))
                        ),
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

                    // 解析K线数据 - 转换为Kline结构体
                    let _klines_1m = match klines_1m_result {
                        Ok(Ok(data)) => data
                            .iter()
                            .map(|candle| Kline {
                                timestamp: candle[0] as i64,
                                open: candle[1],
                                high: candle[2],
                                low: candle[3],
                                close: candle[4],
                                volume: candle[5],
                                quote_volume: if candle.len() > 6 { candle[6] } else { 0.0 },
                                taker_buy_volume: if candle.len() > 7 { candle[7] } else { 0.0 },
                                taker_buy_quote_volume: if candle.len() > 8 {
                                    candle[8]
                                } else {
                                    0.0
                                },
                            })
                            .collect::<Vec<_>>(),
                        _ => {
                            warn!("⚠️  获取{}1mK线失败,跳过启动信号检测", symbol);
                            continue;
                        }
                    };

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
                                quote_volume: if candle.len() > 6 { candle[6] } else { 0.0 },
                                taker_buy_volume: if candle.len() > 7 { candle[7] } else { 0.0 },
                                taker_buy_quote_volume: if candle.len() > 8 {
                                    candle[8]
                                } else {
                                    0.0
                                },
                            })
                            .collect::<Vec<_>>(),
                        _ => {
                            warn!("⚠️  获取{}5mK线失败,跳过启动信号检测", symbol);
                            continue;
                        }
                    };

                    let klines_15m = match klines_15m_result {
                        Ok(Ok(data)) => data
                            .iter()
                            .map(|candle| Kline {
                                timestamp: candle[0] as i64,
                                open: candle[1],
                                high: candle[2],
                                low: candle[3],
                                close: candle[4],
                                volume: candle[5],
                                quote_volume: if candle.len() > 6 { candle[6] } else { 0.0 },
                                taker_buy_volume: if candle.len() > 7 { candle[7] } else { 0.0 },
                                taker_buy_quote_volume: if candle.len() > 8 {
                                    candle[8]
                                } else {
                                    0.0
                                },
                            })
                            .collect::<Vec<_>>(),
                        _ => {
                            warn!("⚠️  获取{}15mK线失败,跳过启动信号检测", symbol);
                            continue;
                        }
                    };

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
                                quote_volume: if candle.len() > 6 { candle[6] } else { 0.0 },
                                taker_buy_volume: if candle.len() > 7 { candle[7] } else { 0.0 },
                                taker_buy_quote_volume: if candle.len() > 8 {
                                    candle[8]
                                } else {
                                    0.0
                                },
                            })
                            .collect::<Vec<_>>(),
                        _ => {
                            warn!("⚠️  获取{}1hK线失败,跳过启动信号检测", symbol);
                            continue;
                        }
                    };

                    // 检测启动信号
                    let staged_manager_read = self.staged_manager.read().await;
                    let position_opt = staged_manager_read.positions.get(&symbol).cloned();
                    drop(staged_manager_read);

                    if let Some(position) = position_opt {
                        // 获取当前价格
                        let current_price = match self.exchange.get_current_price(&symbol).await {
                            Ok(price) => price,
                            Err(e) => {
                                warn!("⚠️  获取{}当前价格失败: {}", symbol, e);
                                continue;
                            }
                        };

                        match self.launch_detector.detect_launch_signal(
                            &klines_5m,
                            &klines_15m,
                            &klines_1h,
                            position.trial_entry_price,
                            current_price,
                        ) {
                            Ok(launch_signal) => {
                                info!(
                                    "🚀 启动信号检测: 5m={} | 15m={} | 1h={} | 1m偏离={:.2}% | 全部确认={} | 得分={:.0}",
                                    launch_signal.m5_signal,
                                    launch_signal.m15_trend,
                                    launch_signal.h1_breakout,
                                    launch_signal.m1_deviation,
                                    launch_signal.all_confirmed,
                                    launch_signal.score
                                );
                                info!("   理由: {}", launch_signal.reason);

                                // 判断是否应该补仓
                                let staged_manager_read = self.staged_manager.read().await;
                                let should_add = staged_manager_read
                                    .should_add_position(&symbol, &launch_signal)
                                    .unwrap_or(false);
                                drop(staged_manager_read);

                                if should_add {
                                    info!("✅ 启动信号全部确认,准备执行70%补仓");

                                    // 获取当前价格
                                    let current_price =
                                        match self.exchange.get_current_price(&symbol).await {
                                            Ok(price) => price,
                                            Err(e) => {
                                                error!("❌ 获取{}当前价格失败: {}", symbol, e);
                                                continue;
                                            }
                                        };

                                    // 执行补仓 - 传入 available_usdt 和 leverage
                                    let mut staged_manager = self.staged_manager.write().await;

                                    // 获取试探持仓配置信息
                                    let (available_usdt, leverage) =
                                        if let Some(_pos) = staged_manager.positions.get(&symbol) {
                                            // 从现有持仓推算原始配置 (简化版: 使用默认值)
                                            (self.max_position_usdt, self.max_leverage as f64)
                                        } else {
                                            (self.max_position_usdt, self.max_leverage as f64)
                                        };

                                    match staged_manager.execute_add_position(
                                        &symbol,
                                        current_price,
                                        available_usdt,
                                        leverage,
                                    ) {
                                        Ok(_) => {
                                            info!("✅ 70%补仓执行成功");
                                            info!(
                                                "   试探入场: ${:.4}",
                                                staged_manager
                                                    .positions
                                                    .get(&symbol)
                                                    .unwrap()
                                                    .trial_entry_price
                                            );
                                            info!(
                                                "   补仓入场: ${:.4}",
                                                staged_manager
                                                    .positions
                                                    .get(&symbol)
                                                    .unwrap()
                                                    .add_entry_price
                                            );
                                            info!(
                                                "   平均成本: ${:.4}",
                                                staged_manager
                                                    .positions
                                                    .get(&symbol)
                                                    .unwrap()
                                                    .avg_cost
                                            );
                                            info!(
                                                "   总仓位: {:.6}",
                                                staged_manager
                                                    .positions
                                                    .get(&symbol)
                                                    .unwrap()
                                                    .total_quantity
                                            );

                                            // ✅ 补仓成功后,同步更新 position_trackers 中的数量
                                            let mut trackers = self.position_trackers.write().await;
                                            if let Some(tracker) = trackers.get_mut(&symbol) {
                                                let new_quantity = staged_manager
                                                    .positions
                                                    .get(&symbol)
                                                    .map(|p| p.total_quantity)
                                                    .unwrap_or(tracker.quantity);
                                                let new_entry_price = staged_manager
                                                    .positions
                                                    .get(&symbol)
                                                    .map(|p| p.avg_cost)
                                                    .unwrap_or(tracker.entry_price);

                                                tracker.quantity = new_quantity;
                                                tracker.entry_price = new_entry_price;
                                                info!(
                                                    "✅ 已同步tracker: 数量{:.6} → 成本${:.4}",
                                                    new_quantity, new_entry_price
                                                );
                                            }
                                            drop(trackers);
                                        }
                                        Err(e) => {
                                            error!("❌ 70%补仓执行失败: {}", e);
                                        }
                                    }
                                } else {
                                    info!("⏸️  启动信号未全部确认,继续等待");
                                }
                            }
                            Err(e) => {
                                warn!("⚠️  启动信号检测失败: {}", e);
                            }
                        }
                    }
                }
            }

            // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
            // 【新增】第二步: 检查分批持仓的快速止损
            // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
            {
                let staged_manager = self.staged_manager.read().await;
                let all_positions: Vec<String> = staged_manager.positions.keys().cloned().collect();
                drop(staged_manager);

                for symbol in all_positions {
                    let current_price = match self.exchange.get_current_price(&symbol).await {
                        Ok(price) => price,
                        Err(e) => {
                            warn!("⚠️  获取{}当前价格失败: {}", symbol, e);
                            continue;
                        }
                    };

                    // 获取持仓时长 - trial_entry_time 是 i64 毫秒时间戳
                    let staged_manager_read = self.staged_manager.read().await;
                    let duration_hours =
                        if let Some(position) = staged_manager_read.positions.get(&symbol) {
                            let now_ms = Utc::now().timestamp_millis();
                            let duration_ms = now_ms - position.trial_entry_time;
                            (duration_ms as f64) / 3600000.0 // 毫秒转小时
                        } else {
                            0.0
                        };
                    drop(staged_manager_read);

                    let staged_manager = self.staged_manager.read().await;
                    match staged_manager.check_stop_loss(&symbol, current_price, duration_hours) {
                        Ok(Some(reason)) => {
                            info!("🚨 {} 触发快速止损: {}", symbol, reason);

                            // 获取持仓信息并clone所需字段
                            let (_side, _quantity) =
                                if let Some(position) = staged_manager.positions.get(&symbol) {
                                    (position.side.clone(), position.total_quantity)
                                } else {
                                    drop(staged_manager);
                                    continue;
                                };

                            // 执行平仓
                            drop(staged_manager);
                            match self.close_position_fully_with_retry(&symbol, 3).await {
                                Ok(_) => info!("✅ 快速止损平仓成功: {}", symbol),
                                Err(e) => {
                                    error!("❌ 快速止损平仓失败: {}", e);
                                    self.send_critical_alert(
                                        &symbol,
                                        &format!("快速止损执行失败: {} - {}", reason, e),
                                    )
                                    .await;
                                }
                            }
                        }
                        Ok(None) => {
                            drop(staged_manager);

                            // ✅ 即使不触发硬性止损,也让AI评估是否应该动态止盈
                            let staged_snapshot = {
                                let staged_manager_read = self.staged_manager.read().await;
                                staged_manager_read.positions.get(&symbol).cloned()
                            };

                            let Some(position) = staged_snapshot else {
                                continue;
                            };

                            let side = position.side.clone();
                            let entry_price = position.avg_cost;
                            let quantity = position.total_quantity;
                            let entry_time =
                                Self::timestamp_ms_to_datetime(position.trial_entry_time);
                            let duration = (Utc::now() - entry_time).num_minutes() as f64 / 60.0;
                            let profit_pct = if side == "LONG" {
                                ((current_price - entry_price) / entry_price) * 100.0
                            } else {
                                ((entry_price - current_price) / entry_price) * 100.0
                            };

                            // ⚙️ 硬性止损规则：仅在严重亏损时触发，其他情况交给AI动态评估
                            let is_meme = Self::is_meme_coin(&symbol);
                            let mut forced_stop_reason: Option<String> = None;

                            // MEME币严格止损：60分钟且亏损超过2%
                            if is_meme && duration >= 1.0 && profit_pct <= -2.0 {
                                forced_stop_reason =
                                    Some("MEME币60分钟亏损超过2%，触发硬性止损".to_string());
                            }
                            // MEME币极端时间止损：持仓超过2小时
                            else if is_meme && duration >= 2.0 {
                                forced_stop_reason =
                                    Some("MEME币持仓超过2小时，触发时间止损".to_string());
                            }
                            // 普通币时间+亏损止损：2小时且亏损超过3%
                            else if !is_meme && duration >= 2.0 && profit_pct <= -3.0 {
                                forced_stop_reason =
                                    Some("持仓超过2小时且亏损3%，触发保守退出".to_string());
                            }
                            // 普通币极端时间止损：持仓超过4小时且未盈利
                            else if !is_meme && duration >= 4.0 && profit_pct <= 0.0 {
                                forced_stop_reason =
                                    Some("持仓超过4小时未盈利，触发保守退出".to_string());
                            }

                            // 极端亏损止损（不分币种）
                            if profit_pct <= -5.0 {
                                forced_stop_reason =
                                    Some("亏损超过5%，触发极端防守止损".to_string());
                            }

                            // 快速止损：30分钟亏损超过3%（防止急速下跌）
                            if duration >= 0.5 && profit_pct <= -3.0 {
                                forced_stop_reason =
                                    Some("30分钟亏损超过3%，触发快速止损".to_string());
                            }

                            if let Some(reason) = forced_stop_reason {
                                info!("🚨 {} 硬性止损触发: {}", symbol, reason);
                                match self.close_position_fully_with_retry(&symbol, 3).await {
                                    Ok(_) => info!("✅ 硬性止损平仓成功，移除持仓记录"),
                                    Err(e) => {
                                        error!("❌ 硬性止损平仓失败: {}", e);
                                        self.send_critical_alert(
                                            &symbol,
                                            &format!("硬性止损执行失败: {} - {}", reason, e),
                                        )
                                        .await;
                                    }
                                }
                                continue;
                            }

                            info!(
                                "🤖 {} 分批持仓AI评估: 盈亏{:+.2}%, 时长{:.1}h",
                                symbol, profit_pct, duration
                            );

                            // ✅ Bug Fix: 分批持仓无止损单追踪，计算默认止损价
                            let stop_loss_price = if side == "LONG" {
                                entry_price * 0.95 // 多单默认止损-5%
                            } else {
                                entry_price * 1.05 // 空单默认止损+5%
                            };

                            match self
                                .evaluate_position_with_ai(
                                    &symbol,
                                    &side,
                                    entry_price,
                                    stop_loss_price,
                                    current_price,
                                    quantity,
                                    duration,
                                    None,
                                    None,
                                )
                                .await
                            {
                                Ok(Some(PositionAction::FullClose {
                                    symbol: close_symbol,
                                    side: _,
                                    quantity: _,
                                    ..
                                })) => {
                                    match self
                                        .close_position_fully_with_retry(&close_symbol, 3)
                                        .await
                                    {
                                        Ok(_) => {
                                            info!("✅ 分批持仓AI平仓成功: {}", close_symbol);
                                        }
                                        Err(e) => {
                                            error!("❌ 分批持仓AI平仓失败: {}", e);
                                            self.send_critical_alert(
                                                &close_symbol,
                                                &format!("分批持仓AI建议全平但执行失败: {}", e),
                                            )
                                            .await;
                                        }
                                    }
                                }
                                Ok(Some(PositionAction::PartialClose {
                                    symbol: close_symbol,
                                    side: _close_side,
                                    close_quantity: _close_quantity,
                                    close_pct,
                                    ..
                                })) => {
                                    info!(
                                        "📉 分批持仓AI建议部分平仓 {} ({}%)",
                                        close_symbol, close_pct
                                    );
                                    match self
                                        .close_position_partially(&close_symbol, close_pct)
                                        .await
                                    {
                                        Ok(remaining_qty) => {
                                            let mut staged_manager =
                                                self.staged_manager.write().await;
                                            if let Some(position) =
                                                staged_manager.positions.get_mut(&close_symbol)
                                            {
                                                position.total_quantity = remaining_qty.max(0.0);
                                                info!(
                                                    "✅ 分批持仓数量已同步: {:.6}",
                                                    position.total_quantity
                                                );
                                                if position.total_quantity <= 0.0001 {
                                                    staged_manager.positions.remove(&close_symbol);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            error!("❌ 分批持仓AI部分平仓失败: {}", e);
                                        }
                                    }
                                }
                                Ok(Some(PositionAction::SetLimitOrder { .. })) => {
                                    warn!("⚠️  分批持仓暂不支持AI限价止盈同步,保持持仓");
                                }
                                Ok(Some(PositionAction::Remove(_))) => {}
                                Ok(None) => {}
                                Err(e) => warn!("⚠️  分批持仓AI评估失败: {}", e),
                            }
                        }
                        Err(e) => {
                            warn!("⚠️  {} 止损检查失败: {}", symbol, e);
                            drop(staged_manager);
                        }
                    }
                }
            }

            // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
            // 【原有逻辑】第三步: 检查旧的position_trackers (保持兼容)
            // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
            // 【修复】无论 tracker_snapshots 是否为空，都应该获取真实持仓
            // 这样可以确保手动建仓或程序重启后的持仓也能正确管理
            let exchange_positions = match self.exchange.get_positions().await {
                Ok(pos) => pos,
                Err(e) => {
                    warn!("⚠️  获取持仓列表失败: {}", e);
                    warn!("🔍 错误详情: {:?}", e);
                    // ✅ Bug Fix: 即使获取失败也使用空vec,不能跳过小仓位保护逻辑
                    Vec::new()
                }
            };

            // 【单持仓保护】保证金<0.5U + 亏损 + 资金费率不利时平掉小仓位
            debug!(
                "🔍 开始小仓位保护检查: 共{}个持仓",
                exchange_positions.len()
            );
            for position in &exchange_positions {
                let symbol = position.symbol.clone();

                let symbol_rules = match self.exchange.get_symbol_trading_rules(&symbol).await {
                    Ok(rules) => rules,
                    Err(e) => {
                        warn!("⚠️  {} 获取交易规则失败: {}", symbol, e);
                        continue;
                    }
                };

                if position.size.abs() < symbol_rules.min_qty {
                    warn!(
                        "⚠️  {} 持仓数量 {:.8} 小于最小交易数量 {:.8}，无法通过 API 平仓，跳过处理",
                        symbol,
                        position.size.abs(),
                        symbol_rules.min_qty
                    );
                    let mut trackers = self.position_trackers.write().await;
                    trackers.remove(&symbol);
                    info!("✅ {} 已从追踪器中移除（尘埃持仓）", symbol);
                    continue;
                }

                debug!("   检查持仓: {} size={:.8}", symbol, position.size);
                if position.size <= f64::EPSILON {
                    debug!("   {} 仓位过小,跳过", symbol);
                    continue;
                }

                // 根据方向恢复带符号仓位, 便于统一判断与资金费率同向
                let is_long = if position.side.eq_ignore_ascii_case("LONG") {
                    true
                } else if position.side.eq_ignore_ascii_case("SHORT") {
                    false
                } else {
                    warn!(
                        "⚠️ {} 未知持仓方向({}), 跳过单仓保护",
                        symbol,
                        position.side.as_str()
                    );
                    continue;
                };
                let signed_size = if is_long {
                    position.size
                } else {
                    -position.size
                };

                if position.entry_price <= 0.0 {
                    warn!(
                        "⚠️ {} 入场价异常({:.4}), 跳过单仓保护",
                        symbol, position.entry_price
                    );
                    continue;
                }

                let notional = signed_size.abs() * position.entry_price;
                let margin = notional / 15.0; // 固定15x杠杆

                debug!(
                    "   {} 保证金计算: notional=${:.2}, margin=${:.4}",
                    symbol, notional, margin
                );

                // 条件1: 保证金 < 0.5U
                if margin >= 0.5 {
                    debug!("   {} 保证金{:.4}U >= 0.5U,不触发保护", symbol, margin);
                    continue;
                }

                info!("✅ {} 符合保证金条件: {:.4}U < 0.5U", symbol, margin);

                // 条件2: 根据盈亏状态决定是否平仓
                let should_close = if position.pnl < 0.0 {
                    // 亏损中: 直接平仓,不管资金费率
                    info!(
                        "🚨 {} 小仓位亏损保护触发: 保证金{:.2}U, PnL={:.4}, 方向={}",
                        symbol,
                        margin,
                        position.pnl,
                        position.side.as_str()
                    );
                    true
                } else {
                    // 盈利中: 只有资金费率不利时才平仓
                    let funding_rate = match self.exchange.get_funding_rate(&symbol).await {
                        Ok((rate, _, _, _, _)) => rate,
                        Err(e) => {
                            warn!("⚠️ {} 获取资金费率失败: {}", symbol, e);
                            continue;
                        }
                    };

                    let unfavorable_funding =
                        (is_long && funding_rate > 0.0) || (!is_long && funding_rate < 0.0);

                    if unfavorable_funding {
                        info!(
                            "🚨 {} 小仓位盈利+资金费率不利保护触发: 保证金{:.2}U, PnL={:.4}, 资金费率={:.4}%, 方向={}",
                            symbol,
                            margin,
                            position.pnl,
                            funding_rate * 100.0,
                            position.side.as_str()
                        );
                    }

                    unfavorable_funding
                };

                if !should_close {
                    continue;
                }

                error!(
                    "🚨 {} 执行小仓位保护平仓: 保证金{:.2}U, PnL={:.4}, 方向={}",
                    symbol,
                    margin,
                    position.pnl,
                    position.side.as_str()
                );

                match self.close_position_fully_with_retry(&symbol, 3).await {
                    Ok(_) => {
                        self.send_critical_alert(
                            &symbol,
                            "小仓位亏损+资金费率不利,执行保护平仓...",
                        )
                        .await;
                    }
                    Err(e) => {
                        error!("❌ {} 小仓位亏损保护平仓失败: {}", symbol, e);
                        self.send_critical_alert(
                            &symbol,
                            "小仓位亏损+资金费率不利,执行保护平仓...",
                        )
                        .await;
                    }
                }
            }

            // 如果没有 tracker 记录但有真实持仓，跳过后续的 AI 分析逻辑（防止误操作）
            // 但至少持仓数据已经同步到前端了
            if tracker_snapshots.is_empty() {
                continue;
            }

            let mut actions_to_execute = Vec::new();
            let mut batch_inputs = Vec::new();
            let mut batch_contexts: HashMap<String, PreparedPositionContext> = HashMap::new();

            for snapshot in tracker_snapshots.values() {
                let symbol = snapshot.symbol.clone();
                let side = snapshot.side.clone();
                let entry_price = snapshot.entry_price;
                let entry_time = snapshot.entry_time;
                let quantity = snapshot.quantity;

                // 获取当前持仓
                let maybe_position = exchange_positions.iter().find(|p| p.symbol == symbol);

                // 如果持仓不存在,说明已被止损/止盈触发
                if maybe_position.is_none() {
                    info!("✅ {} 持仓已平仓(止损/止盈触发)", symbol);
                    actions_to_execute.push(PositionAction::Remove(symbol));
                    continue;
                }

                let position = maybe_position.unwrap();
                let current_price = position.mark_price;
                let live_quantity = position.size.abs();

                let notional_value = live_quantity * current_price;
                const MIN_NOTIONAL: f64 = 1.0;

                if notional_value < MIN_NOTIONAL {
                    warn!(
                        "⚠️  {} 发现尘埃持仓 (数量={:.8}, 价格=${:.4}, 价值=${:.4}), 视为已平仓并清理",
                        symbol, live_quantity, current_price, notional_value
                    );
                    actions_to_execute.push(PositionAction::Remove(symbol.clone()));
                    continue;
                }

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

                // 【P0-3】5分钟快速止损 - 入场失败立即退出
                let duration_minutes = (Utc::now() - entry_time).num_minutes();
                if duration_minutes < 5 && profit_pct < -0.5 {
                    warn!(
                        "🚨 {} 5分钟法则触发: 持仓{}分钟亏损{:.2}%, 入场失败立即止损",
                        symbol, duration_minutes, profit_pct
                    );
                    actions_to_execute.push(PositionAction::FullClose {
                        symbol,
                        side,
                        quantity,
                        reason: "entry_failure_5min".to_string(),
                    });
                    continue; // 跳过AI评估
                }

                // 【P1-2】快速止损 - 持仓>30分钟且亏损>3%时触发 (加快风控响应)
                if duration >= 0.5 && profit_pct < -3.0 {
                    warn!(
                        "🚨 {} 快速止损触发: {}分钟亏损{:+.2}%, 执行全仓止损",
                        symbol,
                        (duration * 60.0) as i32,
                        profit_pct
                    );
                    actions_to_execute.push(PositionAction::FullClose {
                        symbol,
                        side,
                        quantity,
                        reason: format!("quick_stop_loss_-3pct_{}min", (duration * 60.0) as i32),
                    });
                    continue; // 跳过后续处理,直接执行止损
                }

                // 【极端止损】持仓亏损超过-5%强制平仓 (保护本金)
                if profit_pct < -5.0 {
                    warn!(
                        "🚨 {} 亏损超过-5%({:+.2}%),执行极端止损",
                        symbol, profit_pct
                    );
                    actions_to_execute.push(PositionAction::FullClose {
                        symbol,
                        side,
                        quantity,
                        reason: "extreme_loss".to_string(),
                    });
                    continue;
                }

                // 【AI 动态止盈评估】使用Gemini单个评估(不再批量)
                let stop_loss_price = if let Some(ref sl_id) = snapshot.stop_loss_order_id {
                    match self.exchange.get_order_status_detail(&symbol, sl_id).await {
                        Ok(status) => status.stop_price.unwrap_or(entry_price),
                        Err(_) => entry_price,
                    }
                } else {
                    if side == "LONG" {
                        entry_price * 0.95
                    } else {
                        entry_price * 1.05
                    }
                };

                match self
                    .prepare_position_ai_context(
                        &symbol,
                        &side,
                        entry_price,
                        stop_loss_price,
                        current_price,
                        quantity,
                        duration,
                        snapshot.stop_loss_order_id.clone(),
                        snapshot.take_profit_order_id.clone(),
                    )
                    .await
                {
                    Ok(PositionEvaluationStep::Immediate(action)) => {
                        actions_to_execute.push(action);
                    }
                    Ok(PositionEvaluationStep::Context(ctx)) => {
                        batch_inputs.push(ctx.to_batch_input());
                        batch_contexts.insert(ctx.symbol.clone(), ctx);
                    }
                    Ok(PositionEvaluationStep::Skip) => {}
                    Err(e) => {
                        warn!("⚠️  {} 准备AI评估上下文失败: {}", symbol, e);
                    }
                }
            }

            if !batch_inputs.is_empty() {
                match self.gemini.evaluate_positions_batch(batch_inputs).await {
                    Ok(results) => {
                        for (symbol, decision) in results {
                            if let Some(ctx) = batch_contexts.remove(&symbol) {
                                match self.handle_position_ai_decision(&ctx, &decision).await {
                                    Ok(Some(action)) => actions_to_execute.push(action),
                                    Ok(None) => {}
                                    Err(e) => {
                                        warn!("⚠️  {} 应用AI决策失败: {}", symbol, e);
                                    }
                                }
                            } else {
                                warn!("⚠️  批量AI返回未知symbol {}, 可能上下文已被移除", symbol);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("⚠️  Gemini 批量评估失败: {}", e);
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
                        side: _,
                        quantity: _,
                        reason,
                    } => match self.close_position_fully_with_retry(&symbol, 3).await {
                        Ok(_) => {
                            symbols_to_remove.push(symbol);
                        }
                        Err(e) => {
                            error!("❌ 全部平仓失败({}): {}", reason, e);
                            self.send_critical_alert(
                                &symbol,
                                &format!("全部平仓失败({}): {}", reason, e),
                            )
                            .await;
                        }
                    },
                    PositionAction::PartialClose {
                        symbol,
                        side,
                        close_quantity,
                        close_pct,
                        entry_price: _,
                        stop_loss_price, // ✅ Bug Fix: 使用原始止损价格
                        remaining_quantity: _,
                        stop_loss_order_id: _,
                    } => {
                        info!(
                            "📉 执行部分平仓: {} 计划数量 {:.6} ({}%)",
                            symbol, close_quantity, close_pct
                        );
                        let canceled_orders = match self.cancel_symbol_trigger_orders(&symbol).await
                        {
                            Ok(ids) => ids,
                            Err(e) => {
                                warn!("⚠️  清理旧触发单失败: {}", e);
                                Vec::new()
                            }
                        };
                        info!(
                            "🗑️ 取消旧触发单 {} 个: {:?}",
                            canceled_orders.len(),
                            canceled_orders
                        );

                        let actual_remaining =
                            match self.close_position_partially(&symbol, close_pct).await {
                                Ok(remaining) => remaining,
                                Err(e) => {
                                    error!("❌ 部分平仓失败: {}", e);
                                    continue;
                                }
                            };

                        if actual_remaining > f64::EPSILON {
                            match self
                                .exchange
                                .set_stop_loss(
                                    &symbol,
                                    &side,
                                    actual_remaining,
                                    stop_loss_price,
                                    None,
                                )
                                .await
                            {
                                Ok(new_sl_id) => {
                                    tracker_mutations.push(TrackerMutation::QuantityAndStopLoss {
                                        symbol,
                                        new_quantity: actual_remaining,
                                        new_stop_loss_order_id: Some(new_sl_id),
                                    });
                                    info!("✅ 止损已根据实际剩余数量更新: {:.6}", actual_remaining);
                                }
                                Err(e) => {
                                    warn!("⚠️  根据实际剩余数量移动止损失败: {}", e);
                                    tracker_mutations.push(TrackerMutation::QuantityAndStopLoss {
                                        symbol,
                                        new_quantity: actual_remaining,
                                        new_stop_loss_order_id: None,
                                    });
                                }
                            }
                        } else {
                            info!("✅ {} 部分平仓后已无剩余仓位，清理追踪器", symbol);
                            symbols_to_remove.push(symbol);
                        }
                    }
                    PositionAction::SetLimitOrder {
                        symbol,
                        side,
                        quantity,
                        limit_price,
                        take_profit_order_id: _,
                    } => {
                        let canceled_orders = match self.cancel_symbol_trigger_orders(&symbol).await
                        {
                            Ok(ids) => ids,
                            Err(e) => {
                                warn!("⚠️  清理旧触发单失败: {}", e);
                                Vec::new()
                            }
                        };
                        info!(
                            "🗑️ 取消旧触发单 {} 个: {:?}",
                            canceled_orders.len(),
                            canceled_orders
                        );

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

    /// 定时重新分析延迟开仓队列 - 每3.5分钟检查是否有合适的入场机会
    pub async fn reanalyze_pending_entries(self: Arc<Self>) {
        info!("🔄 延迟开仓队列重新分析线程已启动");

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(210)).await; // 3.5分钟

            // 获取队列快照
            let pending_snapshot = {
                let pending = self.pending_entries.read().await;
                pending.clone()
            };

            if pending_snapshot.is_empty() {
                continue;
            }

            info!(
                "🔍 开始重新分析延迟开仓队列，当前有 {} 个币种待处理",
                pending_snapshot.len()
            );

            let mut symbols_to_remove = Vec::new();

            for (symbol, mut entry) in pending_snapshot {
                let now = Utc::now();
                let elapsed_hours = (now - entry.first_signal_time).num_hours();

                // 检查退出条件1: 超过6小时未有新信号
                if elapsed_hours >= 6 {
                    info!("⏰ {} 已超过6小时，从延迟队列移除", symbol);
                    symbols_to_remove.push(symbol.clone());
                    continue;
                }

                // 检查退出条件2: 是否已有持仓（可能在其他地方已开仓）
                {
                    let trackers = self.position_trackers.read().await;
                    if trackers.contains_key(&symbol) {
                        info!("✅ {} 已开仓，从延迟队列移除", symbol);
                        symbols_to_remove.push(symbol.clone());
                        continue;
                    }
                }

                // 检查退出条件3: 是否收到资金出逃信号
                let mut fund_escape_signal = false;
                {
                    let coins = self.tracked_coins.read().await;
                    if let Some(alert) = coins.get(&entry.alert.coin) {
                        if alert.alert_type == AlertType::FundEscape {
                            fund_escape_signal = true;
                        }
                    }
                }

                if fund_escape_signal {
                    let detection_time = entry.fund_escape_detected_at.unwrap_or_else(|| {
                        entry.fund_escape_detected_at = Some(now);
                        now
                    });
                    let elapsed_seconds = (now - detection_time).num_seconds();
                    let waited_minutes = elapsed_seconds / 60;
                    info!(
                        "🚨 {} 检测到资金出逃信号,将在10分钟后移除 (已等待{}分钟)",
                        symbol, waited_minutes
                    );

                    if elapsed_seconds >= 600 {
                        info!("🚨 {} 资金出逃信号持续超过10分钟，执行移除", symbol);
                        symbols_to_remove.push(symbol.clone());
                    }

                    // 更新fund_escape_detected_at到队列，方便后续宽限判断
                    let mut pending = self.pending_entries.write().await;
                    if let Some(existing) = pending.get_mut(&symbol) {
                        existing.fund_escape_detected_at = entry.fund_escape_detected_at;
                    }
                } else {
                    // 当前无资金出逃信号，清空历史记录
                    entry.fund_escape_detected_at = None;
                }

                // 更新重试次数和时间
                entry.retry_count += 1;
                entry.last_analysis_time = now;

                info!(
                    "🔄 重新分析延迟开仓币种: {} (第{}次重试，首次信号时间: {})",
                    symbol,
                    entry.retry_count,
                    entry.first_signal_time.format("%H:%M:%S")
                );

                // 重新执行AI分析（复用 analyze_and_trade 的逻辑）
                if let Err(e) = self.analyze_and_trade(entry.alert.clone()).await {
                    warn!("⚠️  {} 重新分析失败: {}", symbol, e);
                }

                // 更新队列中的重试次数
                let mut pending = self.pending_entries.write().await;
                if let Some(existing) = pending.get_mut(&symbol) {
                    existing.retry_count = entry.retry_count;
                    existing.last_analysis_time = entry.last_analysis_time;
                    existing.fund_escape_detected_at = entry.fund_escape_detected_at;
                }
                drop(pending);
            }

            // 批量移除已完成的币种
            if !symbols_to_remove.is_empty() {
                let mut pending = self.pending_entries.write().await;
                for symbol in symbols_to_remove {
                    pending.remove(&symbol);
                }
                info!("📊 延迟开仓队列清理完成，剩余 {} 个币种", pending.len());
            }
        }
    }

    /// 复用AI评估逻辑，统一对持仓做动态处理
    async fn prepare_position_ai_context(
        &self,
        symbol: &str,
        side: &str,
        entry_price: f64,
        stop_loss_price: f64,
        current_price: f64,
        quantity: f64,
        duration: f64,
        stop_loss_order_id: Option<String>,
        take_profit_order_id: Option<String>,
    ) -> Result<PositionEvaluationStep> {
        let profit_pct = if side == "LONG" {
            ((current_price - entry_price) / entry_price) * 100.0
        } else {
            ((entry_price - current_price) / entry_price) * 100.0
        };

        if profit_pct >= 15.0 {
            info!(
                "💰 {} 盈利已达 {:+.2}% >= 15%, 触发强制全仓平仓 (锁定利润)",
                symbol, profit_pct
            );
            return Ok(PositionEvaluationStep::Immediate(
                PositionAction::FullClose {
                    symbol: symbol.to_string(),
                    side: side.to_string(),
                    quantity,
                    reason: "profit_target_15pct".to_string(),
                },
            ));
        }

        if profit_pct >= 10.0 && duration >= 2.0 {
            info!(
                "⏰ {} 盈利 {:+.2}% >= 10% 且持仓 {:.1}h >= 2h, 触发强制全仓平仓 (时间效率)",
                symbol, profit_pct, duration
            );
            return Ok(PositionEvaluationStep::Immediate(
                PositionAction::FullClose {
                    symbol: symbol.to_string(),
                    side: side.to_string(),
                    quantity,
                    reason: "profit_time_optimization".to_string(),
                },
            ));
        }

        info!(
            "🤖 {} 当前盈亏 {:+.2}%, 调用 AI 评估持仓管理...",
            symbol, profit_pct
        );

        let market_context = match self.collect_position_market_context(symbol).await? {
            Some(ctx) => ctx,
            None => return Ok(PositionEvaluationStep::Skip),
        };

        if market_context.klines_1h.len() >= 3 {
            let last_3_candles = &market_context.klines_1h[market_context.klines_1h.len() - 3..];
            let all_opposite = if side == "LONG" {
                last_3_candles.iter().all(|k| k.close < k.open)
            } else {
                last_3_candles.iter().all(|k| k.close > k.open)
            };

            if all_opposite {
                let opposite_type = if side == "LONG" { "阴线" } else { "阳线" };
                let close_pct = if profit_pct >= 10.0 {
                    70.0
                } else if profit_pct >= 5.0 {
                    60.0
                } else {
                    50.0
                };

                warn!(
                    "📉 {} 触发P0-1规则: 连续3根1h{} (Valuescan止盈信号)",
                    symbol, opposite_type
                );
                warn!(
                    "   持仓方向: {} | 当前盈亏: {:+.2}% | 建议止盈: {:.0}%",
                    side, profit_pct, close_pct
                );

                let close_quantity = (quantity * (close_pct / 100.0)).clamp(0.0, quantity);
                let trading_rules = self.exchange.get_symbol_trading_rules(symbol).await?;
                let min_notional = trading_rules.min_notional.unwrap_or(5.0);

                let market_price = match self.exchange.get_current_price(symbol).await {
                    Ok(price) => price,
                    Err(_) => entry_price,
                };

                let position_total_value = quantity * market_price;
                let suggested_close_value = close_quantity * market_price;

                if suggested_close_value < min_notional {
                    let min_ratio_pct = (min_notional / position_total_value * 100.0).ceil();

                    if min_ratio_pct <= 100.0 {
                        let adjusted_close_pct = min_ratio_pct;
                        let adjusted_close_qty = quantity * (adjusted_close_pct / 100.0);
                        let adjusted_remaining = (quantity - adjusted_close_qty).max(0.0);

                        warn!(
                            "⚠️ {} 部分平仓比率调整: AI建议{:.0}% (${:.2}) → 实际执行{:.0}% (${:.2})，满足MIN_NOTIONAL ${:.0}",
                            symbol, close_pct, suggested_close_value, adjusted_close_pct, adjusted_close_qty * market_price, min_notional
                        );

                        return Ok(PositionEvaluationStep::Immediate(
                            PositionAction::PartialClose {
                                symbol: symbol.to_string(),
                                side: side.to_string(),
                                close_quantity: adjusted_close_qty,
                                close_pct: adjusted_close_pct,
                                entry_price,
                                stop_loss_price,
                                remaining_quantity: adjusted_remaining,
                                stop_loss_order_id: stop_loss_order_id.clone(),
                            },
                        ));
                    } else {
                        warn!(
                            "⚠️ {} 持仓总价值(${:.2}) < MIN_NOTIONAL(${:.0})，无法部分平仓，执行全部平仓",
                            symbol, position_total_value, min_notional
                        );
                        return Ok(PositionEvaluationStep::Immediate(
                            PositionAction::FullClose {
                                symbol: symbol.to_string(),
                                side: side.to_string(),
                                quantity,
                                reason: "valuescan_p0_1_min_notional_full_close".to_string(),
                            },
                        ));
                    }
                }

                let remaining_quantity = (quantity - close_quantity).max(0.0);
                return Ok(PositionEvaluationStep::Immediate(
                    PositionAction::PartialClose {
                        symbol: symbol.to_string(),
                        side: side.to_string(),
                        close_quantity,
                        close_pct,
                        entry_price,
                        stop_loss_price,
                        remaining_quantity,
                        stop_loss_order_id: stop_loss_order_id.clone(),
                    },
                ));
            }
        }

        let is_meme = Self::is_meme_coin(symbol);
        let time_limit_hours = if is_meme { 4.0 } else { 8.0 };

        if duration >= time_limit_hours {
            warn!(
                "⏰ {} 触发P0-2规则: 持仓{:.1}h >= {:.0}h ({}流动性时间窗口)",
                symbol,
                duration,
                time_limit_hours,
                if is_meme { "MEME币" } else { "山寨币" }
            );
            warn!("   Valuescan核心理论: 流动性最多维持4-8h, 超时强制退出");

            return Ok(PositionEvaluationStep::Immediate(
                PositionAction::FullClose {
                    symbol: symbol.to_string(),
                    side: side.to_string(),
                    quantity,
                    reason: format!("time_limit_{}h", time_limit_hours as u32),
                },
            ));
        }

        if market_context.klines_1h.len() >= 2 {
            let current_candle = &market_context.klines_1h[market_context.klines_1h.len() - 1];
            let prev_candle = &market_context.klines_1h[market_context.klines_1h.len() - 2];

            let current_body = (current_candle.close - current_candle.open).abs();
            let prev_body = (prev_candle.close - prev_candle.open).abs();

            let is_rebound = if side == "LONG" {
                current_candle.close > current_candle.open
            } else {
                current_candle.close < current_candle.open
            };

            if is_rebound && prev_body > 0.0 && current_body > prev_body * 0.5 {
                let rebound_strength_pct = (current_body / prev_body) * 100.0;
                info!(
                    "💪 {} P1-1信号: 反弹力度{:.1}% (>50% 强支撑/护盘)",
                    symbol, rebound_strength_pct
                );
            }
        }

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

        let support_klines_5m = convert_to_support_klines(&market_context.klines_5m);
        let support_klines_15m = convert_to_support_klines(&market_context.klines_15m);
        let support_klines_1h = convert_to_support_klines(&market_context.klines_1h);

        let support_analyzer = SupportAnalyzer::new();
        let support_analysis = match support_analyzer.analyze_supports(
            &support_klines_5m,
            &support_klines_15m,
            &support_klines_1h,
            current_price,
            entry_price,
            market_context.indicators.sma_20,
            market_context.indicators.sma_50,
            market_context.indicators.bb_lower,
            market_context.indicators.bb_middle,
        ) {
            Ok(analysis) => analysis,
            Err(e) => {
                warn!("⚠️  {} 支撑位分析失败: {}", symbol, e);
                return Ok(PositionEvaluationStep::Skip);
            }
        };
        let support_text = support_analyzer.format_support_analysis(&support_analysis);

        let last_5m_close = match market_context.klines_5m.last() {
            Some(k) => k.close,
            None => {
                warn!("⚠️  {} 5mK线数据为空", symbol);
                return Ok(PositionEvaluationStep::Skip);
            }
        };
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

        let stop_loss_lookup_id = stop_loss_order_id.clone();
        let take_profit_lookup_id = take_profit_order_id.clone();
        let (current_stop_loss, current_take_profit) = {
            let stop_loss_price = if let Some(sl_id) = stop_loss_lookup_id.as_ref() {
                match self.exchange.get_order_status_detail(symbol, sl_id).await {
                    Ok(order) => order.stop_price,
                    Err(e) => {
                        warn!(
                            "⚠️  查询止损挂单失败: symbol={} sl_id={} err={}",
                            symbol, sl_id, e
                        );
                        None
                    }
                }
            } else {
                None
            };

            let take_profit_price = if let Some(tp_id) = take_profit_lookup_id.as_ref() {
                match self.exchange.get_order_status_detail(symbol, tp_id).await {
                    Ok(order) => Some(order.price),
                    Err(e) => {
                        warn!(
                            "⚠️  查询止盈挂单失败: symbol={} tp_id={} err={}",
                            symbol, tp_id, e
                        );
                        None
                    }
                }
            } else {
                None
            };

            (stop_loss_price, take_profit_price)
        };

        Ok(PositionEvaluationStep::Context(PreparedPositionContext {
            symbol: symbol.to_string(),
            side: side.to_string(),
            entry_price,
            stop_loss_price,
            current_price,
            quantity,
            duration,
            profit_pct,
            stop_loss_order_id,
            take_profit_order_id,
            market: market_context,
            support_text,
            deviation_desc,
            current_stop_loss,
            current_take_profit,
        }))
    }

    async fn handle_position_ai_decision(
        &self,
        ctx: &PreparedPositionContext,
        ai_decision: &PositionManagementDecision,
    ) -> Result<Option<PositionAction>> {
        info!(
            "🎯 AI 决策: {} | 理由: {} | 盈利潜力: {} | 置信度: {}",
            ai_decision.action,
            ai_decision.reason,
            ai_decision.profit_potential,
            ai_decision.confidence
        );

        let confidence_value = Self::map_confidence_to_score(&ai_decision.confidence);
        let decision_text = format!(
            "{} | 盈利潜力: {} | 置信度: {}",
            ai_decision.action, ai_decision.profit_potential, ai_decision.confidence
        );
        let signal_type = Self::normalize_signal_type(&ai_decision.action);
        let ai_record = AiAnalysisRecord {
            id: None,
            timestamp: Utc::now().to_rfc3339(),
            symbol: ctx.symbol.clone(),
            decision: decision_text,
            confidence: confidence_value,
            signal_type: Some(signal_type.to_string()),
            reason: ai_decision.reason.clone(),
            valuescan_score: None,
            risk_reward_ratio: None,
            entry_price: None,
            stop_loss: None,
            resistance: None,
            support: None,
        };

        if let Err(e) = self.db.insert_ai_analysis(&ai_record) {
            warn!("⚠️  保存AI持仓分析到数据库失败: {}", e);
        }

        let action = match ai_decision.action.as_str() {
            "HOLD" => {
                info!("✅ AI 建议继续持有 {}", ctx.symbol);
                None
            }
            "PARTIAL_CLOSE" => {
                if let Some(close_pct) = ai_decision.close_percentage {
                    info!("📉 AI 建议部分平仓 {} ({}%)", ctx.symbol, close_pct);
                    let close_quantity =
                        (ctx.quantity * (close_pct / 100.0)).clamp(0.0, ctx.quantity);
                    let remaining_quantity = (ctx.quantity - close_quantity).max(0.0);

                    if close_quantity <= f64::EPSILON {
                        warn!("⚠️  计算得到的平仓数量过小, 跳过本次部分平仓");
                        None
                    } else {
                        let trading_rules =
                            self.exchange.get_symbol_trading_rules(&ctx.symbol).await?;
                        let min_notional = trading_rules.min_notional.unwrap_or(5.0);

                        let market_price = match self.exchange.get_current_price(&ctx.symbol).await
                        {
                            Ok(price) => price,
                            Err(_) => ctx.entry_price,
                        };

                        let position_total_value = ctx.quantity * market_price;
                        let suggested_close_value = close_quantity * market_price;

                        if suggested_close_value < min_notional {
                            let min_ratio_pct =
                                (min_notional / position_total_value * 100.0).ceil();

                            if min_ratio_pct <= 100.0 {
                                let adjusted_close_pct = min_ratio_pct;
                                let adjusted_close_qty =
                                    ctx.quantity * (adjusted_close_pct / 100.0);
                                let adjusted_remaining =
                                    (ctx.quantity - adjusted_close_qty).max(0.0);

                                warn!(
                                    "⚠️ {} 部分平仓比率调整: AI建议{:.0}% (${:.2}) → 实际执行{:.0}% (${:.2})，满足MIN_NOTIONAL ${:.0}",
                                    ctx.symbol,
                                    close_pct,
                                    suggested_close_value,
                                    adjusted_close_pct,
                                    adjusted_close_qty * market_price,
                                    min_notional
                                );

                                Some(PositionAction::PartialClose {
                                    symbol: ctx.symbol.clone(),
                                    side: ctx.side.clone(),
                                    close_quantity: adjusted_close_qty,
                                    close_pct: adjusted_close_pct,
                                    entry_price: ctx.entry_price,
                                    stop_loss_price: ctx.stop_loss_price,
                                    remaining_quantity: adjusted_remaining,
                                    stop_loss_order_id: ctx.stop_loss_order_id.clone(),
                                })
                            } else {
                                warn!(
                                    "⚠️ {} 持仓总价值(${:.2}) < MIN_NOTIONAL(${:.0})，无法部分平仓，执行全部平仓",
                                    ctx.symbol, position_total_value, min_notional
                                );
                                Some(PositionAction::FullClose {
                                    symbol: ctx.symbol.clone(),
                                    side: ctx.side.clone(),
                                    quantity: ctx.quantity,
                                    reason: "min_notional_full_close".to_string(),
                                })
                            }
                        } else {
                            Some(PositionAction::PartialClose {
                                symbol: ctx.symbol.clone(),
                                side: ctx.side.clone(),
                                close_quantity,
                                close_pct,
                                entry_price: ctx.entry_price,
                                stop_loss_price: ctx.stop_loss_price,
                                remaining_quantity,
                                stop_loss_order_id: ctx.stop_loss_order_id.clone(),
                            })
                        }
                    }
                } else {
                    warn!(
                        "⚠️ {} PARTIAL_CLOSE缺少close_percentage字段, 跳过",
                        ctx.symbol
                    );
                    None
                }
            }
            "FULL_CLOSE" => {
                info!("🚨 AI 建议全部平仓 {}", ctx.symbol);
                Some(PositionAction::FullClose {
                    symbol: ctx.symbol.clone(),
                    side: ctx.side.clone(),
                    quantity: ctx.quantity,
                    reason: "ai_decision".to_string(),
                })
            }
            "SET_LIMIT_ORDER" => {
                if let Some(limit_price) = ai_decision.limit_price {
                    info!(
                        "🎯 AI 建议设置限价止盈单 {} @ ${:.4}",
                        ctx.symbol, limit_price
                    );
                    Some(PositionAction::SetLimitOrder {
                        symbol: ctx.symbol.clone(),
                        side: ctx.side.clone(),
                        quantity: ctx.quantity,
                        limit_price,
                        take_profit_order_id: ctx.take_profit_order_id.clone(),
                    })
                } else {
                    warn!("⚠️  AI 建议设置限价单但未提供价格,保持持仓");
                    None
                }
            }
            other => {
                warn!("⚠️  未知的 AI 决策动作: {}, 保持持仓", other);
                None
            }
        };

        Ok(action)
    }

    async fn evaluate_position_with_ai(
        &self,
        symbol: &str,
        side: &str,
        entry_price: f64,
        stop_loss_price: f64, // ✅ Bug Fix: 止损价格参数
        current_price: f64,
        quantity: f64,
        duration: f64,
        stop_loss_order_id: Option<String>,
        take_profit_order_id: Option<String>,
    ) -> Result<Option<PositionAction>> {
        match self
            .prepare_position_ai_context(
                symbol,
                side,
                entry_price,
                stop_loss_price,
                current_price,
                quantity,
                duration,
                stop_loss_order_id,
                take_profit_order_id,
            )
            .await?
        {
            PositionEvaluationStep::Skip => Ok(None),
            PositionEvaluationStep::Immediate(action) => Ok(Some(action)),
            PositionEvaluationStep::Context(ctx) => {
                let ctx_ref = ctx.to_prompt_context();
                let prompt = build_position_prompt_v2(&ctx_ref);

                let ai_decision_result = tokio::time::timeout(
                    tokio::time::Duration::from_secs(180),
                    self.gemini.analyze_position_management(&prompt),
                )
                .await;

                let ai_decision = match ai_decision_result {
                    Ok(Ok(decision)) => decision,
                    Ok(Err(e)) => {
                        error!("❌ AI持仓评估失败: {}, 保持持仓", e);
                        return Ok(None);
                    }
                    Err(_) => {
                        warn!("⚠️  AI持仓评估超时, 保持持仓");
                        return Ok(None);
                    }
                };

                self.handle_position_ai_decision(&ctx, &ai_decision).await
            }
        }
    }

    /// 根据增强版AI分析返回的推荐动作顺序执行
    #[allow(dead_code)]
    async fn execute_recommended_actions(
        &self,
        analysis: &EnhancedPositionAnalysis,
        current_symbol: &str,
    ) -> Result<Vec<String>> {
        fn normalize_sides(side: Option<&String>) -> (Option<String>, Option<String>) {
            side.map(|value| {
                let normalized = value.trim().to_uppercase();
                match normalized.as_str() {
                    "LONG" => (Some("BUY".to_string()), Some("LONG".to_string())),
                    "SHORT" => (Some("SELL".to_string()), Some("SHORT".to_string())),
                    "BUY" => (Some("BUY".to_string()), Some("LONG".to_string())),
                    "SELL" => (Some("SELL".to_string()), Some("SHORT".to_string())),
                    _ => (Some(normalized.clone()), Some(normalized)),
                }
            })
            .unwrap_or((None, None))
        }

        fn parse_order_ids(raw: Option<&String>) -> Vec<String> {
            raw.map(|ids| {
                ids.split(|c| c == ',' || c == '|' || c == ';')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
        }

        if analysis.recommended_actions.is_empty() {
            return Ok(Vec::new());
        }

        let mut actions = analysis.recommended_actions.clone();
        actions.sort_by(|a, b| a.priority.cmp(&b.priority));

        let mut results = Vec::with_capacity(actions.len());

        for action in actions {
            let action_type = action.action_type.clone();
            let reason = action.reason.clone();
            let ActionParams {
                symbol,
                side,
                quantity,
                price,
                stop_loss,
                take_profit,
                auto_set_protection: _,
                trigger_price,
                order_id,
            } = action.params;

            let symbol = symbol
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| current_symbol.to_string());

            let outcome: Result<String> = match action_type.as_str() {
                "IMMEDIATE_CLOSE" => {
                    let qty = quantity.ok_or_else(|| anyhow::anyhow!("立即平仓缺少 quantity"))?;
                    let (_, position_side) = normalize_sides(side.as_ref());
                    let position_side =
                        position_side.ok_or_else(|| anyhow::anyhow!("立即平仓缺少持仓方向"))?;

                    warn!("⚠️ 立即平仓: {} - {}", symbol, reason);
                    if let Err(e) = self.close_position_fully_with_retry(&symbol, 3).await {
                        self.send_critical_alert(
                            &symbol,
                            &format!("立即平仓失败 ({}): {}", reason, e),
                        )
                        .await;
                        return Err(e);
                    }

                    Ok(format!(
                        "⚠️ 立即平仓完成: {} {} 数量 {:.4}",
                        symbol, position_side, qty
                    ))
                }
                "LIMIT_ORDER" => {
                    let qty = quantity.ok_or_else(|| anyhow::anyhow!("限价单缺少 quantity"))?;
                    let px = price.ok_or_else(|| anyhow::anyhow!("限价单缺少 price"))?;
                    let (order_side, position_side) = normalize_sides(side.as_ref());
                    let order_side =
                        order_side.ok_or_else(|| anyhow::anyhow!("限价单缺少交易方向"))?;

                    let order_id = self
                        .exchange
                        .limit_order(
                            &symbol,
                            qty,
                            &order_side,
                            px,
                            position_side.as_deref(),
                            false,
                        )
                        .await?;
                    info!("📝 限价单已挂: {} {} @ {:.4}", symbol, order_side, px);

                    let attachments = if stop_loss.is_some() || take_profit.is_some() {
                        let pos_side = position_side
                            .clone()
                            .ok_or_else(|| anyhow::anyhow!("设置止盈止损缺少 positionSide"))?;
                        self.order_manager
                            .place_protection_orders(
                                &symbol,
                                &pos_side,
                                qty,
                                stop_loss,
                                take_profit,
                            )
                            .await?
                    } else {
                        Vec::new()
                    };

                    let mut message = format!(
                        "📝 限价单已挂: {} {} @ {:.4} (order_id={})",
                        symbol, order_side, px, order_id
                    );
                    if !attachments.is_empty() {
                        message.push_str(&format!(" | {}", attachments.join(", ")));
                    }
                    Ok(message)
                }
                "TRIGGER_ORDER" => {
                    let qty = quantity.ok_or_else(|| anyhow::anyhow!("触发单缺少 quantity"))?;
                    let trigger =
                        trigger_price.ok_or_else(|| anyhow::anyhow!("触发单缺少 trigger_price"))?;
                    let (_, position_side) = normalize_sides(side.as_ref());
                    let position_side =
                        position_side.ok_or_else(|| anyhow::anyhow!("触发单缺少 position_side"))?;

                    // 默认使用市价触发 + 开仓动作，后续可扩展 CLOSE/其他类型
                    let mut action = "OPEN".to_string();
                    let mut smart_close_hint: Option<String> = None;

                    // 智能平仓: 若存在同方向持仓, 根据触发价与当前价决定是否自动 CLOSE
                    match self.exchange.get_positions().await {
                        Ok(positions) => {
                            if let Some(position) = positions
                                .into_iter()
                                .find(|p| p.symbol == symbol && p.size.abs() > f64::EPSILON)
                            {
                                if position.side.eq_ignore_ascii_case(&position_side) {
                                    match self.exchange.get_current_price(&symbol).await {
                                        Ok(current_price) => {
                                            let (reason_label, should_close) =
                                                match position.side.as_str() {
                                                    "LONG" => {
                                                        if trigger < current_price {
                                                            ("LONG 持仓止损判定", true)
                                                        } else if trigger > current_price {
                                                            ("LONG 持仓止盈判定", true)
                                                        } else {
                                                            ("LONG 持仓价位触发", true)
                                                        }
                                                    }
                                                    "SHORT" => {
                                                        if trigger > current_price {
                                                            ("SHORT 持仓止损判定", true)
                                                        } else if trigger < current_price {
                                                            ("SHORT 持仓止盈判定", true)
                                                        } else {
                                                            ("SHORT 持仓价位触发", true)
                                                        }
                                                    }
                                                    _ => ("", false),
                                                };

                                            if should_close {
                                                action = "CLOSE".to_string();
                                                smart_close_hint = Some(format!(
                                                    "{}: 当前价={:.4} → 触发价={:.4}",
                                                    reason_label, current_price, trigger
                                                ));
                                            }
                                        }
                                        Err(err) => {
                                            warn!(
                                                "⚠️  获取{}当前价失败(触发单智能判定): {}",
                                                symbol, err
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            warn!("⚠️  获取{}持仓失败(触发单智能判定): {}", symbol, err);
                        }
                    }

                    let volatility = match self.calculate_volatility(&symbol).await {
                        Ok(value) => value,
                        Err(err) => {
                            warn!(
                                "⚠️  计算{}波动率失败: {}，使用默认值 {:.2}%",
                                symbol, err, DEFAULT_VOLATILITY_PERCENT
                            );
                            DEFAULT_VOLATILITY_PERCENT
                        }
                    };

                    let (trigger_type, limit_price_adjusted): (&str, Option<f64>) =
                        if let Some(limit) = price {
                            info!("📊 AI 指定限价 {:.4}, 使用 STOP 限价触发单", limit);
                            ("STOP", Some(limit))
                        } else if volatility > 3.0 {
                            info!("📊 市场波动率 {:.2}% (高),使用 STOP_MARKET", volatility);
                            ("STOP_MARKET", None)
                        } else if volatility < 1.0 {
                            info!("📊 市场波动率 {:.2}% (低),使用 STOP 限价单", volatility);
                            let buffer = if position_side == "LONG" {
                                1.002
                            } else {
                                0.998
                            };
                            ("STOP", Some(trigger * buffer))
                        } else {
                            info!("📊 市场波动率 {:.2}% (中),使用 STOP_MARKET", volatility);
                            ("STOP_MARKET", None)
                        };

                    let order_id = self
                        .exchange
                        .place_trigger_order(
                            &symbol,
                            trigger_type,
                            &action,
                            &position_side,
                            qty,
                            trigger,
                            limit_price_adjusted,
                        )
                        .await?;

                    if let Some(hint) = &smart_close_hint {
                        info!("🤖 智能平仓判定: {}", hint);
                    }

                    info!(
                        "🎯 触发单已设: {} {} {} @ trigger={:.4} (type={}, order_id={})",
                        symbol, action, position_side, trigger, trigger_type, order_id
                    );

                    {
                        let mut orders = self.active_trigger_orders.lock().await;
                        orders.push(TriggerOrderRecord {
                            order_id: order_id.clone(),
                            symbol: symbol.clone(),
                            position_side: position_side.clone(),
                            trigger_price: trigger,
                            action: action.clone(),
                            created_at: Utc::now(),
                            reason: reason.clone(),
                        });
                    }
                    info!(
                        "📒 已加入触发单监控: {} {} {} (order_id={})",
                        symbol, action, position_side, order_id
                    );

                    let mut message = format!(
                        "🎯 触发单已设: {} {} {} @ {:.4} (order_id={})",
                        symbol, action, position_side, trigger, order_id
                    );
                    if let Some(hint) = smart_close_hint {
                        message.push_str(&format!(" | {}", hint));
                    }
                    Ok(message)
                }
                "CANCEL_TRIGGER" => {
                    let order_id = order_id
                        .as_deref()
                        .ok_or_else(|| anyhow::anyhow!("取消触发单缺少 order_id"))?
                        .to_string();
                    self.order_manager.cancel_order(&symbol, &order_id).await?;
                    {
                        let mut orders = self.active_trigger_orders.lock().await;
                        let before = orders.len();
                        orders.retain(|record| record.order_id != order_id);
                        if before != orders.len() {
                            info!("🗂️ 已从触发单监控移除: {}", order_id);
                        }
                    }
                    info!("❌ 已取消触发单: {}", order_id);
                    Ok(format!("❌ 已取消触发单: {}", order_id))
                }
                "SET_STOP_LOSS_TAKE_PROFIT" => {
                    let qty =
                        quantity.ok_or_else(|| anyhow::anyhow!("设置止盈止损缺少 quantity"))?;
                    let (_, position_side) = normalize_sides(side.as_ref());
                    let position_side = position_side
                        .ok_or_else(|| anyhow::anyhow!("设置止盈止损缺少 positionSide"))?;

                    let canceled_orders = match self.cancel_symbol_trigger_orders(&symbol).await {
                        Ok(ids) => ids,
                        Err(e) => {
                            warn!("⚠️  清理旧触发单失败: {}", e);
                            Vec::new()
                        }
                    };
                    info!(
                        "🗑️ 取消旧触发单 {} 个: {:?}",
                        canceled_orders.len(),
                        canceled_orders
                    );

                    let mut updates = Vec::new();
                    if let Some(stop_loss) = stop_loss {
                        let order_id = self
                            .exchange
                            .set_stop_loss(&symbol, &position_side, qty, stop_loss, None)
                            .await?;
                        updates.push(format!("SL {:.4}#{}", stop_loss, order_id));
                    }
                    if let Some(take_profit) = take_profit {
                        let order_id = self
                            .exchange
                            .set_limit_take_profit(&symbol, &position_side, qty, take_profit)
                            .await?;
                        updates.push(format!("TP {:.4}#{}", take_profit, order_id));
                    }

                    if updates.is_empty() {
                        return Err(anyhow::anyhow!("未提供止损或止盈参数"));
                    }

                    info!("🛡️ 止盈止损已更新: {}", updates.join(", "));
                    Ok(format!(
                        "🛡️ 止盈止损已更新: {} -> {}",
                        symbol,
                        updates.join(", ")
                    ))
                }
                "CANCEL_STOP_LOSS_TAKE_PROFIT" => {
                    let order_ids = parse_order_ids(order_id.as_ref());
                    if order_ids.is_empty() {
                        return Err(anyhow::anyhow!("取消止盈止损缺少 order_id"));
                    }
                    self.order_manager
                        .cancel_orders_batch(&symbol, &order_ids)
                        .await?;
                    info!("🗑️ 已取消止盈止损单: {}", order_ids.join(", "));
                    Ok(format!("🗑️ 已取消止盈止损单: {}", order_ids.join(", ")))
                }
                other => Err(anyhow::anyhow!(format!("未知动作类型: {}", other))),
            };

            match outcome {
                Ok(message) => results.push(message),
                Err(err) => {
                    let error_msg = format!("❌ 执行动作失败 [{}]: {}", action_type, err);
                    warn!("{}", error_msg);
                    results.push(error_msg);
                }
            }
        }

        Ok(results)
    }

    /// 取消指定币种已登记的止损/止盈触发单，防止重复堆积
    async fn cancel_symbol_trigger_orders(&self, symbol: &str) -> Result<Vec<u64>> {
        // 先快照当前 tracker，避免持锁执行异步请求
        let tracker_snapshot = {
            let trackers = self.position_trackers.read().await;
            trackers.get(symbol).cloned()
        };

        let Some(tracker) = tracker_snapshot else {
            return Ok(Vec::new());
        };

        let mut targets: Vec<(&str, String)> = Vec::new();
        if let Some(order_id) = tracker.stop_loss_order_id.clone() {
            targets.push(("止损", order_id));
        }
        if let Some(order_id) = tracker.take_profit_order_id.clone() {
            targets.push(("止盈", order_id));
        }

        if targets.is_empty() {
            return Ok(Vec::new());
        }

        let mut canceled_raw: Vec<String> = Vec::new();

        for (order_type, order_id) in targets {
            match self.exchange.cancel_order(symbol, &order_id).await {
                Ok(_) => {
                    info!("🧹 {} 旧{}单已取消: {}", symbol, order_type, order_id);
                    canceled_raw.push(order_id);
                }
                Err(err) => {
                    warn!(
                        "⚠️  {} 旧{}单取消失败 (order_id={}): {}",
                        symbol, order_type, order_id, err
                    );
                }
            }
        }

        if canceled_raw.is_empty() {
            return Ok(Vec::new());
        }

        {
            let mut trackers = self.position_trackers.write().await;
            if let Some(tracker) = trackers.get_mut(symbol) {
                if tracker
                    .stop_loss_order_id
                    .as_deref()
                    .map(|id| canceled_raw.iter().any(|raw| raw == id))
                    .unwrap_or(false)
                {
                    tracker.stop_loss_order_id = None;
                }
                if tracker
                    .take_profit_order_id
                    .as_deref()
                    .map(|id| canceled_raw.iter().any(|raw| raw == id))
                    .unwrap_or(false)
                {
                    tracker.take_profit_order_id = None;
                }
                tracker.last_check_time = Utc::now();
            }
        }

        let mut canceled_numeric = Vec::new();
        for raw in canceled_raw {
            match raw.parse::<u64>() {
                Ok(id) => canceled_numeric.push(id),
                Err(_) => {
                    warn!(
                        "⚠️  order_id 无法转换为数字 (symbol={}, raw={})，仍视为已清理",
                        symbol, raw
                    );
                }
            }
        }

        Ok(canceled_numeric)
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

    /// 定期校准持仓追踪状态，避免数量漂移
    async fn sync_position_trackers(&self) -> Result<()> {
        let positions = self.exchange.get_positions().await?;
        let mut synced = 0;
        let mut removed = 0;

        let mut trackers = self.position_trackers.write().await;
        let mut exchange_symbols: HashSet<String> = HashSet::new();

        for pos in positions.iter() {
            exchange_symbols.insert(pos.symbol.clone());
            if let Some(tracker) = trackers.get_mut(&pos.symbol) {
                let real_qty = pos.size.abs();
                if (tracker.quantity - real_qty).abs() > 0.0001 {
                    warn!(
                        "⚠️  {} tracker 偏差: 本地 {:.8} vs 实际 {:.8}, 已修正",
                        pos.symbol, tracker.quantity, real_qty
                    );
                    tracker.quantity = real_qty;
                    tracker.last_check_time = Utc::now();
                    synced += 1;
                }
            }
        }

        trackers.retain(|symbol, _| {
            let exists = exchange_symbols.contains(symbol);
            if !exists {
                warn!("⚠️  {} 已平仓但 tracker 仍存在,已清理", symbol);
                removed += 1;
            }
            exists
        });

        if synced > 0 || removed > 0 {
            info!("🔄 Tracker 同步完成: 修正 {}, 清理 {}", synced, removed);
        } else {
            debug!("Tracker 同步: 未检测到偏差");
        }

        Ok(())
    }

    /// 检查止盈止损互斥: 当一方订单成交(FILLED)时,自动取消另一方
    async fn check_sl_tp_mutual_exclusion(&self) -> Result<()> {
        // 获取所有tracker的快照
        let trackers_snapshot: Vec<(String, Option<String>, Option<String>)> = {
            let trackers = self.position_trackers.read().await;
            trackers
                .iter()
                .filter(|(_, t)| t.stop_loss_order_id.is_some() || t.take_profit_order_id.is_some())
                .map(|(symbol, t)| {
                    (
                        symbol.clone(),
                        t.stop_loss_order_id.clone(),
                        t.take_profit_order_id.clone(),
                    )
                })
                .collect()
        };

        if trackers_snapshot.is_empty() {
            return Ok(());
        }

        let mut mutations: Vec<(String, Option<String>, Option<String>)> = Vec::new();

        for (symbol, sl_order_id, tp_order_id) in trackers_snapshot {
            let mut new_sl_id = sl_order_id.clone();
            let mut new_tp_id = tp_order_id.clone();
            let mut sl_filled = false;
            let mut tp_filled = false;

            // 检查止损单状态
            if let Some(ref sl_id) = sl_order_id {
                match self.exchange.get_order_status(&symbol, sl_id).await {
                    Ok(status) => {
                        if status == "FILLED" || status == "EXPIRED" || status == "CANCELED" {
                            sl_filled = status == "FILLED";
                            new_sl_id = None;
                            if sl_filled {
                                info!("🔴 {} 止损单已成交: {}", symbol, sl_id);
                            }
                        }
                    }
                    Err(e) => {
                        // 订单可能已不存在
                        debug!("⚠️ {} 查询止损单状态失败: {}", symbol, e);
                        new_sl_id = None;
                    }
                }
            }

            // 检查止盈单状态
            if let Some(ref tp_id) = tp_order_id {
                match self.exchange.get_order_status(&symbol, tp_id).await {
                    Ok(status) => {
                        if status == "FILLED" || status == "EXPIRED" || status == "CANCELED" {
                            tp_filled = status == "FILLED";
                            new_tp_id = None;
                            if tp_filled {
                                info!("🟢 {} 止盈单已成交: {}", symbol, tp_id);
                            }
                        }
                    }
                    Err(e) => {
                        debug!("⚠️ {} 查询止盈单状态失败: {}", symbol, e);
                        new_tp_id = None;
                    }
                }
            }

            // 互斥处理: 止损成交则取消止盈
            if sl_filled {
                if let Some(ref tp_id) = tp_order_id {
                    match self.order_manager.cancel_order(&symbol, tp_id).await {
                        Ok(_) => info!("✅ {} 止损触发,已取消止盈单: {}", symbol, tp_id),
                        Err(e) => warn!("⚠️ {} 取消止盈单失败: {}", symbol, e),
                    }
                    new_tp_id = None;
                }
            }

            // 互斥处理: 止盈成交则取消止损
            if tp_filled {
                if let Some(ref sl_id) = sl_order_id {
                    match self.order_manager.cancel_order(&symbol, sl_id).await {
                        Ok(_) => info!("✅ {} 止盈触发,已取消止损单: {}", symbol, sl_id),
                        Err(e) => warn!("⚠️ {} 取消止损单失败: {}", symbol, e),
                    }
                    new_sl_id = None;
                }
            }

            // 记录需要更新的tracker
            if new_sl_id != sl_order_id || new_tp_id != tp_order_id {
                mutations.push((symbol, new_sl_id, new_tp_id));
            }
        }

        // 批量更新tracker
        if !mutations.is_empty() {
            let mut trackers = self.position_trackers.write().await;
            for (symbol, new_sl_id, new_tp_id) in mutations {
                if let Some(tracker) = trackers.get_mut(&symbol) {
                    tracker.stop_loss_order_id = new_sl_id;
                    tracker.take_profit_order_id = new_tp_id;
                }
            }
        }

        Ok(())
    }

    /// 清理已经无对应持仓的触发单/减仓单,避免阻塞后续开仓
    async fn cleanup_orphaned_trigger_orders(&self) -> Result<()> {
        info!("⏰ 开始执行定期孤立触发单清理...");

        let positions = self.exchange.get_positions().await?;
        let active_symbols: HashSet<String> = positions
            .iter()
            .filter(|p| p.size.abs() > f64::EPSILON)
            .map(|p| p.symbol.clone())
            .collect();

        // 复制一份快照,避免在持有锁的情况下执行异步调用
        let trackers_snapshot = {
            let trackers = self.position_trackers.read().await;
            trackers.clone()
        };

        let mut cleaned_count = 0usize;
        let mut symbols_to_remove = Vec::new();

        for (symbol, tracker) in trackers_snapshot {
            if active_symbols.contains(&symbol) {
                continue;
            }

            let orphaned_duration = Utc::now() - tracker.entry_time;
            let orphaned_minutes = Duration::num_minutes(&orphaned_duration);
            debug!(
                "⏱️ {} 已空仓 {} 分钟, 开始清理遗留触发单",
                symbol, orphaned_minutes
            );

            if let Some(order_id) = tracker.stop_loss_order_id.as_deref() {
                match self.order_manager.cancel_order(&symbol, order_id).await {
                    Ok(_) => {
                        info!(
                            "🗑️ 清理孤立触发单: {} SL order_id={} (持仓已平仓)",
                            symbol, order_id
                        );
                        cleaned_count += 1;
                    }
                    Err(e) => {
                        warn!(
                            "⚠️ 取消孤立触发单失败: {} SL order_id={} ({})",
                            symbol, order_id, e
                        );
                    }
                }
            }

            if let Some(order_id) = tracker.take_profit_order_id.as_deref() {
                match self.order_manager.cancel_order(&symbol, order_id).await {
                    Ok(_) => {
                        info!(
                            "🗑️ 清理孤立触发单: {} TP order_id={} (持仓已平仓)",
                            symbol, order_id
                        );
                        cleaned_count += 1;
                    }
                    Err(e) => {
                        warn!(
                            "⚠️ 取消孤立触发单失败: {} TP order_id={} ({})",
                            symbol, order_id, e
                        );
                    }
                }
            }

            info!("🗑️ 清理孤立触发单: {} SL/TP (持仓已平仓)", symbol);
            symbols_to_remove.push(symbol);
        }

        if !symbols_to_remove.is_empty() {
            let mut trackers = self.position_trackers.write().await;
            for symbol in symbols_to_remove {
                trackers.remove(&symbol);
            }
        }

        info!("✅ 定期孤立触发单清理完成 (清理 {} 个订单)", cleaned_count);

        Ok(())
    }

    /// 完全平仓
    async fn close_position_fully(&self, symbol: &str) -> Result<()> {
        info!("🔄 准备全仓平仓: {}", symbol);

        // 先快照当前追踪信息，记录交易历史时使用
        let tracker_snapshot = {
            let trackers = self.position_trackers.read().await;
            trackers.get(symbol).cloned()
        };
        let staged_snapshot = {
            let staged = self.staged_manager.read().await;
            staged.positions.get(symbol).cloned()
        };

        // 查询实时仓位，确保使用真实数量与方向
        let positions = self.exchange.get_positions().await?;
        let real_position = positions.into_iter().find(|p| p.symbol == symbol);
        let (real_size, side) = match real_position {
            Some(pos) => {
                if pos.size.abs() < 0.0001 {
                    warn!("⚠️  {} 实际持仓过小 ({:.8}),清理追踪记录", symbol, pos.size);
                    self.clear_position_tracking(symbol).await;
                    return Ok(());
                }
                (pos.size.abs(), pos.side.to_ascii_uppercase())
            }
            None => {
                warn!("⚠️  {} 无持仓,清理追踪记录", symbol);
                self.clear_position_tracking(symbol).await;
                return Ok(());
            }
        };

        info!("📊 {} 实时持仓: {:.8} ({})", symbol, real_size, side);

        // 平仓前先清理保护单，避免 reduceOnly 冲突
        if let Some(tracker) = tracker_snapshot.as_ref() {
            if let Some(sl_id) = &tracker.stop_loss_order_id {
                if let Err(e) = self.order_manager.cancel_order(symbol, sl_id).await {
                    warn!("⚠️  取消 {} 止损单失败: {}", symbol, e);
                }
            }
            if let Some(tp_id) = &tracker.take_profit_order_id {
                if let Err(e) = self.order_manager.cancel_order(symbol, tp_id).await {
                    warn!("⚠️  取消 {} 止盈单失败: {}", symbol, e);
                }
            }
        }

        let exit_price = self.exchange.get_current_price(symbol).await?;

        self.exchange
            .close_position(symbol, &side, real_size)
            .await?;
        info!("✅ {} 全仓平仓成功", symbol);

        self.finalize_position_close(
            symbol,
            &side,
            real_size,
            exit_price,
            tracker_snapshot,
            staged_snapshot,
        )
        .await;

        Ok(())
    }

    /// 带重试的完全平仓，失败时指数退避并最终触发市价单兜底
    async fn close_position_fully_with_retry(&self, symbol: &str, max_retries: u32) -> Result<()> {
        const DEFAULT_MAX_RETRIES: u32 = 3;
        let retries = if max_retries == 0 {
            DEFAULT_MAX_RETRIES
        } else {
            max_retries
        };

        for attempt in 1..=retries {
            match self.close_position_fully(symbol).await {
                Ok(_) => {
                    info!("✅ {} 平仓成功 (尝试 {}/{})", symbol, attempt, retries);
                    return Ok(());
                }
                Err(e) => {
                    if attempt < retries {
                        let backoff_secs = 2_u64.pow(attempt);
                        warn!(
                            "⚠️  {} 平仓失败 (尝试 {}/{}): {} - {}秒后重试",
                            symbol, attempt, retries, e, backoff_secs
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(backoff_secs)).await;
                    } else {
                        error!("❌ {} 多次重试后仍失败: {}", symbol, e);
                        warn!("🔄 最后尝试: 使用市价单强制平仓 {}", symbol);
                        match self.try_market_fallback_close(symbol).await {
                            Ok(_) => {
                                info!("✅ 市价单 fallback 成功: {}", symbol);
                                return Ok(());
                            }
                            Err(fallback_err) => {
                                error!("❌ 市价单 fallback 也失败: {}", fallback_err);
                                return Err(anyhow!(
                                    "平仓完全失败 - 限价单: {} / 市价单: {}",
                                    e,
                                    fallback_err
                                ));
                            }
                        }
                    }
                }
            }
        }

        Err(anyhow!("不应到达此处"))
    }

    /// 写入交易记录并清理追踪器
    async fn finalize_position_close(
        &self,
        symbol: &str,
        side: &str,
        quantity: f64,
        exit_price: f64,
        tracker_snapshot: Option<PositionTracker>,
        staged_snapshot: Option<StagedPosition>,
    ) {
        self.record_trade_history(
            symbol,
            side,
            quantity,
            exit_price,
            tracker_snapshot,
            staged_snapshot,
        )
        .await;
        self.clear_position_tracking(symbol).await;
    }

    /// 移除 position_tracker 与 staged_manager 中的缓存
    async fn clear_position_tracking(&self, symbol: &str) {
        {
            let mut trackers = self.position_trackers.write().await;
            trackers.remove(symbol);
        }
        let mut staged_manager = self.staged_manager.write().await;
        staged_manager.positions.remove(symbol);
    }

    /// 使用市价单 fallback 强制平仓
    async fn try_market_fallback_close(&self, symbol: &str) -> Result<()> {
        warn!("🔄 启动市价单 fallback 强制平仓: {}", symbol);

        let tracker_snapshot = {
            let trackers = self.position_trackers.read().await;
            trackers.get(symbol).cloned()
        };
        let staged_snapshot = {
            let staged = self.staged_manager.read().await;
            staged.positions.get(symbol).cloned()
        };

        // 再次取消保护单，避免残留订单阻塞
        if let Some(tracker) = tracker_snapshot.as_ref() {
            if let Some(sl_id) = &tracker.stop_loss_order_id {
                if let Err(e) = self.order_manager.cancel_order(symbol, sl_id).await {
                    warn!("⚠️  Fallback 取消止损单失败: {}", e);
                }
            }
            if let Some(tp_id) = &tracker.take_profit_order_id {
                if let Err(e) = self.order_manager.cancel_order(symbol, tp_id).await {
                    warn!("⚠️  Fallback 取消止盈单失败: {}", e);
                }
            }
        }

        let positions = match self.exchange.get_positions().await {
            Ok(data) => data,
            Err(fetch_err) => {
                return Err(fetch_err);
            }
        };

        let Some(pos) = positions.iter().find(|p| p.symbol == symbol) else {
            warn!("⚠️  市价单 Fallback 未找到 {} 持仓,自动清理追踪", symbol);
            self.clear_position_tracking(symbol).await;
            return Ok(());
        };

        let fallback_side = pos.side.to_ascii_uppercase();
        let fallback_size = pos.size.abs();
        if fallback_size <= 0.0 {
            warn!(
                "⚠️  市价单 Fallback 检测到 {} 仓位数量为0，直接清理追踪记录",
                symbol
            );
            self.clear_position_tracking(symbol).await;
            return Ok(());
        }

        let exit_price = match self.exchange.get_current_price(symbol).await {
            Ok(price) => price,
            Err(price_err) => {
                warn!(
                    "⚠️  获取 {} 最新价格失败 ({})，使用标记价 {:.4}",
                    symbol, price_err, pos.mark_price
                );
                pos.mark_price
            }
        };

        self.exchange
            .close_position(symbol, &fallback_side, fallback_size)
            .await?;

        info!("✅ 市价单 Fallback 平仓成功: {}", symbol);
        self.finalize_position_close(
            symbol,
            &fallback_side,
            fallback_size,
            exit_price,
            tracker_snapshot,
            staged_snapshot,
        )
        .await;
        Ok(())
    }

    /// 向风险通道发送告警，日志 + Telegram + 独立文件
    async fn send_critical_alert(&self, symbol: &str, reason: &str) {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
        let alert_msg = format!(
            "🚨 紧急告警 - 需人工干预\n\n交易对: {}\n时间: {}\n原因: {}\n\n请立即检查持仓状态!",
            symbol, timestamp, reason
        );

        error!("🚨 CRITICAL ALERT [{}] {}", symbol, reason);
        error!("{}", alert_msg);

        if let Some(bot) = &self.telegram_bot {
            match env::var("TELEGRAM_ALERT_CHAT_ID") {
                Ok(chat_id) => match chat_id.parse::<i64>() {
                    Ok(chat_id_i64) => {
                        let chat = teloxide::types::ChatId(chat_id_i64);
                        if let Err(e) = bot.send_message(chat, &alert_msg).await {
                            error!("❌ Telegram 告警发送失败: {}", e);
                        } else {
                            info!("✅ Telegram 告警已发送");
                        }
                    }
                    Err(e) => warn!("⚠️ TELEGRAM_ALERT_CHAT_ID 解析失败: {}", e),
                },
                Err(_) => warn!("⚠️ 未配置 TELEGRAM_ALERT_CHAT_ID, Telegram 告警不可用"),
            }
        }

        if let Err(e) = tokio::fs::create_dir_all("logs").await {
            error!("❌ 创建日志目录失败: {}", e);
        }

        let alert_file = "logs/critical_alerts.log";
        match tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(alert_file)
            .await
        {
            Ok(mut file) => {
                let log_entry = format!("[{}] {}\n{}\n\n", timestamp, symbol, reason);
                if let Err(e) = file.write_all(log_entry.as_bytes()).await {
                    error!("❌ 写入告警日志失败: {}", e);
                }
            }
            Err(e) => error!("❌ 打开告警日志失败: {}", e),
        }
    }

    /// 使用实时仓位信息执行部分平仓，返回剩余仓位
    async fn close_position_partially(&self, symbol: &str, close_pct: f64) -> Result<f64> {
        if close_pct <= 0.0 {
            return Err(anyhow!("{} 部分平仓百分比无效: {}", symbol, close_pct));
        }

        info!("📉 准备部分平仓: {} ({}%)", symbol, close_pct);
        let positions = self.exchange.get_positions().await?;
        let real_position = positions
            .iter()
            .find(|p| p.symbol == symbol)
            .ok_or_else(|| anyhow!("{} 无实时持仓", symbol))?;
        let real_size = real_position.size.abs();

        if real_size <= f64::EPSILON {
            warn!("⚠️  {} 实际持仓数量为零, 直接清理追踪器", symbol);
            self.clear_position_tracking(symbol).await;
            return Ok(0.0);
        }

        let side = if real_position.size > 0.0 {
            "LONG"
        } else {
            "SHORT"
        };
        let pct = close_pct.min(100.0);
        let mut close_amount = real_size * (pct / 100.0);
        if close_amount <= f64::EPSILON {
            return Err(anyhow!(
                "{} 计算部分平仓数量过小: {:.8}",
                symbol,
                close_amount
            ));
        }

        close_amount = close_amount.min(real_size);
        info!(
            "📊 {} 实时持仓: {:.8}, 平仓 {}% -> {:.8}",
            symbol, real_size, pct, close_amount
        );

        if close_amount / real_size > 0.9999 {
            info!("⚠️  {} 计划部分平仓接近全仓，建议直接调用全平逻辑", symbol);
        }

        if let Err(e) = self
            .exchange
            .close_position(symbol, side, close_amount)
            .await
        {
            error!("❌ {} 部分平仓失败: {}", symbol, e);
            if let Ok(updated_positions) = self.exchange.get_positions().await {
                if let Some(updated_pos) = updated_positions.iter().find(|p| p.symbol == symbol) {
                    let mut trackers = self.position_trackers.write().await;
                    if let Some(tracker) = trackers.get_mut(symbol) {
                        tracker.quantity = updated_pos.size.abs();
                        tracker.last_check_time = Utc::now();
                        warn!(
                            "⚠️  平仓失败但已同步 tracker: {} = {:.8}",
                            symbol, tracker.quantity
                        );
                    }
                } else {
                    let mut trackers = self.position_trackers.write().await;
                    trackers.remove(symbol);
                    warn!("⚠️  {} 持仓已消失,清理 tracker", symbol);
                }
            }
            return Err(e);
        }

        info!("✅ {} 部分平仓成功: {:.8}", symbol, close_amount);
        let updated_positions = self.exchange.get_positions().await?;
        let remaining_quantity = updated_positions
            .iter()
            .find(|p| p.symbol == symbol)
            .map(|p| p.size.abs())
            .unwrap_or(0.0);

        {
            let mut trackers = self.position_trackers.write().await;
            if remaining_quantity <= 0.0001 {
                trackers.remove(symbol);
                info!("🗑️  {} 部分平仓后无剩余持仓, 已清理 tracker", symbol);
            } else if let Some(tracker) = trackers.get_mut(symbol) {
                tracker.quantity = remaining_quantity;
                tracker.last_check_time = Utc::now();
                info!("📝 更新 tracker: {} 剩余 {:.8}", symbol, tracker.quantity);
            }
        }

        Ok(remaining_quantity.max(0.0))
    }

    /// 平仓完成后写入数据库交易记录
    async fn record_trade_history(
        &self,
        symbol: &str,
        side: &str,
        quantity: f64,
        exit_price: f64,
        tracker_snapshot: Option<PositionTracker>,
        staged_snapshot: Option<StagedPosition>,
    ) {
        let (entry_price, entry_time) = match tracker_snapshot {
            Some(tracker) => (tracker.entry_price, tracker.entry_time),
            None => {
                if let Some(staged) = staged_snapshot {
                    let entry_time = Self::timestamp_ms_to_datetime(staged.trial_entry_time);
                    let entry_price = if staged.avg_cost > 0.0 {
                        staged.avg_cost
                    } else {
                        staged.trial_entry_price
                    };
                    (entry_price, entry_time)
                } else {
                    warn!("⚠️  未找到 {} 的持仓快照，跳过交易记录写入", symbol);
                    return;
                }
            }
        };

        let exit_time = Utc::now();
        let raw_duration = (exit_time - entry_time).num_seconds();
        let hold_duration = if raw_duration < 0 { 0 } else { raw_duration };

        let direction = if side.eq_ignore_ascii_case("LONG") {
            1.0
        } else {
            -1.0
        };
        let pnl = (exit_price - entry_price) * quantity * direction;
        let pnl_pct = if entry_price.abs() <= f64::EPSILON {
            0.0
        } else {
            ((exit_price - entry_price) / entry_price) * 100.0 * direction
        };

        let entry_time_str = entry_time.to_rfc3339();
        let exit_time_str = exit_time.to_rfc3339();
        let trade_record = DbTradeRecord {
            id: None,
            symbol: symbol.to_string(),
            side: side.to_string(),
            entry_price,
            exit_price,
            quantity,
            pnl,
            pnl_pct,
            entry_time: entry_time_str,
            exit_time: exit_time_str.clone(),
            hold_duration,
            strategy_tag: None,
            notes: None,
            created_at: Some(exit_time_str),
        };

        if let Err(e) = self.db.insert_trade(&trade_record) {
            warn!("⚠️  记录交易历史失败: {}", e);
        }
    }

    /// 将毫秒时间戳安全转换为 UTC 时间
    fn timestamp_ms_to_datetime(ms: i64) -> DateTime<Utc> {
        let secs = ms.div_euclid(1000);
        let nsecs = (ms.rem_euclid(1000) as u32) * 1_000_000;
        DateTime::<Utc>::from_timestamp(secs, nsecs).unwrap_or_else(|| Utc::now())
    }

    /// 启动时同步交易所现有持仓到position_trackers
    pub async fn sync_existing_positions(&self) -> Result<()> {
        info!("🔄 正在恢复启动前已存在的持仓...");

        let positions = self.exchange.get_positions().await?;
        let mut recovered_count = 0;

        let mut trackers = self.position_trackers.write().await;
        for position in positions {
            let quantity = position.size.abs();
            if quantity <= f64::EPSILON {
                continue;
            }

            let now = Utc::now();
            trackers.insert(
                position.symbol.clone(),
                PositionTracker {
                    symbol: position.symbol.clone(),
                    entry_price: position.entry_price,
                    quantity,
                    leverage: self.max_leverage,
                    side: position.side.clone(),
                    stop_loss_order_id: None,
                    take_profit_order_id: None,
                    entry_time: now - Duration::hours(1),
                    last_check_time: now,
                },
            );
            info!(
                "✅ 恢复历史持仓: {}, 方向={}, 数量={:.6}, 入场=${:.4}",
                position.symbol, position.side, quantity, position.entry_price
            );
            recovered_count += 1;
        }

        info!("📊 共恢复 {} 个历史持仓", recovered_count);
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
        let symbol = if alert.coin.ends_with("USDT") {
            alert.coin.clone()
        } else {
            format!("{}USDT", alert.coin)
        };
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
        let _history_prompt = if let Some(perf) = &perf_opt {
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
                    quote_volume: if candle.len() > 6 { candle[6] } else { 0.0 },
                    taker_buy_volume: if candle.len() > 7 { candle[7] } else { 0.0 },
                    taker_buy_quote_volume: if candle.len() > 8 { candle[8] } else { 0.0 },
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
                    quote_volume: if candle.len() > 6 { candle[6] } else { 0.0 },
                    taker_buy_volume: if candle.len() > 7 { candle[7] } else { 0.0 },
                    taker_buy_quote_volume: if candle.len() > 8 { candle[8] } else { 0.0 },
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
                    quote_volume: if candle.len() > 6 { candle[6] } else { 0.0 },
                    taker_buy_volume: if candle.len() > 7 { candle[7] } else { 0.0 },
                    taker_buy_quote_volume: if candle.len() > 8 { candle[8] } else { 0.0 },
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

        // 2. 分析1h主入场区
        info!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("📊 第1步: 分析1h主入场区");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let zone_1h = match self.entry_zone_analyzer.analyze_1h_entry_zone(&klines_1h) {
            Ok(zone) => zone,
            Err(e) => {
                warn!("❌ 1h入场区分析失败: {}", e);
                return Ok(());
            }
        };

        info!(
            "✅ 1h主入场区: 理想价格=${:.4}, 范围=${:.4}-${:.4}, 止损=${:.4}, 信心={:?}",
            zone_1h.ideal_entry,
            zone_1h.entry_range.0,
            zone_1h.entry_range.1,
            zone_1h.stop_loss,
            zone_1h.confidence
        );

        // 3. 分析15m辅助入场区
        info!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("📊 第2步: 分析15m辅助入场区");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let zone_15m = match self
            .entry_zone_analyzer
            .analyze_15m_entry_zone(&klines, &zone_1h)
        {
            Ok(zone) => zone,
            Err(e) => {
                warn!("⚠️  15m辅助区分析失败: {}", e);
                return Ok(());
            }
        };

        info!(
            "✅ 15m辅助区: 理想价格=${:.4}, 范围=${:.4}-${:.4}, 关系={:?}",
            zone_15m.ideal_entry,
            zone_15m.entry_range.0,
            zone_15m.entry_range.1,
            zone_15m.relationship
        );

        // 4. 综合决策入场策略
        info!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("🎯 第3步: 综合决策入场策略");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let current_price = klines.last().unwrap().close;
        let entry_decision =
            self.entry_zone_analyzer
                .decide_entry_strategy(&zone_1h, &zone_15m, current_price);

        info!(
            "🎯 量化决策: 动作={:?}, 价格=${:.4}, 仓位={:.0}%, 止损=${:.4}",
            entry_decision.action,
            entry_decision.price,
            entry_decision.position * 100.0,
            entry_decision.stop_loss
        );
        info!("   量化理由: {}", entry_decision.reason);

        // 4. AI综合判断 (K线形态优先)
        info!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("🤖 第4步: AI综合判断(K线形态优先)");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let alert_type_str = if alert.alert_type == AlertType::FundEscape {
            "资金出逃"
        } else {
            "资金流入"
        };

        let zone_1h_summary = format!(
            "理想价${:.4}, 范围${:.4}-${:.4}, 止损${:.4}, 信心{:?}, 仓位{:.0}%",
            zone_1h.ideal_entry,
            zone_1h.entry_range.0,
            zone_1h.entry_range.1,
            zone_1h.stop_loss,
            zone_1h.confidence,
            zone_1h.suggested_position * 100.0
        );

        let zone_15m_summary = format!(
            "理想价${:.4}, 范围${:.4}-${:.4}, 与1h关系{:?}",
            zone_15m.ideal_entry,
            zone_15m.entry_range.0,
            zone_15m.entry_range.1,
            zone_15m
                .relationship
                .as_ref()
                .map(|r| format!("{:?}", r))
                .unwrap_or("未知".to_string())
        );

        let entry_action_str = format!("{:?}", entry_decision.action);

        let use_valuescan_v2 = *USE_VALUESCAN_V2;
        info!(
            "🤖 Valuescan版本: {} (USE_VALUESCAN_V2={})",
            if use_valuescan_v2 { "V2" } else { "V1" },
            use_valuescan_v2
        );

        // 保存V2扩展数据用于数据库记录
        let mut v2_score: Option<f64> = None;
        let mut v2_risk_reward: Option<f64> = None;
        let mut v2_resistance: Option<f64> = None;
        let mut v2_support: Option<f64> = None;

        let ai_signal: TradingSignal = if use_valuescan_v2 {
            let ctx = EntryPromptContext {
                symbol: &symbol,
                alert_type: alert_type_str,
                alert_message: &alert.raw_message,
                fund_type: &alert.fund_type,
                zone_1h_summary: &zone_1h_summary,
                zone_15m_summary: &zone_15m_summary,
                entry_action: &entry_action_str,
                entry_reason: &entry_decision.reason,
                klines_5m: &klines_5m,
                klines_15m: &klines,
                klines_1h: &klines_1h,
                current_price,
                change_24h: None,
            };

            let prompt = build_entry_prompt_v2(&ctx);

            let ai_decision_result = tokio::time::timeout(
                tokio::time::Duration::from_secs(180),
                self.deepseek.analyze_market_v2(&prompt),
            )
            .await;

            let ai_signal_v2: TradingSignalV2 = match ai_decision_result {
                Ok(Ok(signal)) => signal,
                Ok(Err(e)) => {
                    error!("❌ AI开仓分析失败(V2): {}, 跳过本次交易", e);
                    return Ok(());
                }
                Err(_) => {
                    warn!("⚠️  AI开仓分析超时180s (V2), 跳过本次交易");
                    return Ok(());
                }
            };

            info!(
                "🏅 Valuescan V2评分: {:.1}/10 | 风险收益比: {:.2} | 仓位建议: {:.1}%",
                ai_signal_v2.valuescan_score,
                ai_signal_v2.risk_reward_ratio.unwrap_or(0.0),
                ai_signal_v2.position_size_pct
            );

            // ✅ Bug Fix #1: 处理Optional的key_levels字段
            if let Some(ref key_levels) = ai_signal_v2.key_levels {
                info!(
                    "   V2关键位: 阻力=${:.4} | 支撑=${:.4} | 位置={}",
                    key_levels.resistance, key_levels.support, key_levels.current_position
                );
            } else {
                info!("   V2关键位: AI未提供关键位数据");
            }

            // 【P1-3】提高Valuescan V2评分阈值,过滤低质量信号
            info!(
                "🔎 AI评分详细检查: 分数={:.1}, 阈值=6.5, 动作={:?}, 理由={}",
                ai_signal_v2.valuescan_score,
                ai_signal_v2.signal,
                ai_signal_v2.reason.chars().take(50).collect::<String>()
            );

            if ai_signal_v2.valuescan_score < 6.5 {
                info!(
                    "⏸️ Valuescan V2评分{:.1}不足6.5阈值, 跳过本次交易",
                    ai_signal_v2.valuescan_score
                );
                return Ok(());
            }

            info!("✅ Valuescan V2评分检查通过，准备执行交易逻辑");

            // 保存V2数据
            v2_score = Some(ai_signal_v2.valuescan_score);
            v2_risk_reward = ai_signal_v2.risk_reward_ratio;
            if let Some(ref key_levels) = ai_signal_v2.key_levels {
                v2_resistance = Some(key_levels.resistance);
                v2_support = Some(key_levels.support);
            }

            ai_signal_v2.into()
        } else {
            let ctx = EntryPromptContext {
                symbol: &symbol,
                alert_type: alert_type_str,
                alert_message: &alert.raw_message,
                fund_type: &alert.fund_type,
                zone_1h_summary: &zone_1h_summary,
                zone_15m_summary: &zone_15m_summary,
                entry_action: &entry_action_str,
                entry_reason: &entry_decision.reason,
                klines_5m: &klines_5m,
                klines_15m: &klines,
                klines_1h: &klines_1h,
                current_price,
                change_24h: None,
            };

            let prompt = build_entry_prompt_v1(&ctx);

            let ai_decision_result = tokio::time::timeout(
                tokio::time::Duration::from_secs(180),
                self.gemini.analyze_market(&prompt),
            )
            .await;

            match ai_decision_result {
                Ok(Ok(signal)) => signal,
                Ok(Err(e)) => {
                    error!("❌ AI开仓分析失败: {}, 跳过本次交易", e);
                    return Ok(());
                }
                Err(_) => {
                    warn!("⚠️  AI开仓分析超时180s, 跳过本次交易");
                    return Ok(());
                }
            }
        };

        info!(
            "🎯 AI决策: {} | 信心: {} | 入场价: ${:.4} | 止损: ${:.4}",
            ai_signal.signal,
            ai_signal.confidence,
            ai_signal.entry_price.unwrap_or(current_price),
            ai_signal.stop_loss.unwrap_or(entry_decision.stop_loss)
        );
        info!("   AI理由: {}", ai_signal.reason);

        let normalized_ai_signal = ai_signal.signal.trim().to_ascii_uppercase();

        // ✅ 将AI分析写入数据库，便于前端回溯信号
        let confidence_value = Self::map_confidence_to_score(&ai_signal.confidence);
        let entry_price_value = ai_signal.entry_price.unwrap_or(current_price);
        let stop_loss_value = ai_signal.stop_loss.unwrap_or(entry_decision.stop_loss);
        let decision_text = format!(
            "{} | 入场: ${:.4} | 止损: ${:.4}",
            ai_signal.signal, entry_price_value, stop_loss_value
        );
        let signal_type = Self::normalize_signal_type(&ai_signal.signal);
        let ai_record = AiAnalysisRecord {
            id: None,
            timestamp: Utc::now().to_rfc3339(),
            symbol: symbol.clone(),
            decision: decision_text,
            confidence: confidence_value,
            signal_type: Some(signal_type.to_string()),
            reason: ai_signal.reason.clone(),
            valuescan_score: v2_score,
            risk_reward_ratio: v2_risk_reward,
            entry_price: Some(entry_price_value),
            stop_loss: Some(stop_loss_value),
            resistance: v2_resistance,
            support: v2_support,
        };

        if let Err(e) = self.db.insert_ai_analysis(&ai_record) {
            warn!("⚠️  保存AI分析到数据库失败: {}", e);
        }

        // 根据AI决策过滤 - 只过滤SKIP信号,不再强制过滤资金信号矛盾
        match normalized_ai_signal.as_str() {
            "SKIP" => {
                info!("\n⏸️  AI建议跳过: {}", ai_signal.reason);

                // 加入延迟开仓队列，等待后续重新评估
                let mut pending = self.pending_entries.write().await;
                if let Some(existing) = pending.get_mut(&symbol) {
                    existing.retry_count += 1;
                    existing.last_analysis_time = Utc::now();
                    existing.reject_reason = format!("AI SKIP: {}", ai_signal.reason);
                    let retry_count = existing.retry_count;
                    drop(pending);
                    info!(
                        "📝 {} 已在延迟队列中，更新重试次数: {}",
                        symbol, retry_count
                    );
                } else {
                    pending.insert(
                        symbol.clone(),
                        PendingEntry {
                            symbol: symbol.clone(),
                            first_signal_time: Utc::now(),
                            last_analysis_time: Utc::now(),
                            alert: alert.clone(),
                            reject_reason: format!("AI SKIP: {}", ai_signal.reason),
                            retry_count: 0,
                            fund_escape_detected_at: None,
                        },
                    );
                    drop(pending);
                    info!("📝 已加入延迟开仓队列: {} (AI SKIP)", symbol);
                }

                return Ok(());
            }
            "BUY" | "SELL" => {
                // ✅ AI已综合资金信号+K线形态做出判断,直接执行
                info!(
                    "✅ AI综合判断: {} (资金信号: {})",
                    ai_signal.signal, alert_type_str
                );
            }
            _ => {
                warn!("⚠️  未知AI信号: {}, 跳过", ai_signal.signal);
                return Ok(());
            }
        }

        // 5. 执行试探建仓 (使用AI微调后的价格)
        let final_entry_price = ai_signal.entry_price.unwrap_or(entry_decision.price);
        let side = if normalized_ai_signal.eq_ignore_ascii_case("SELL") {
            "SHORT"
        } else {
            "LONG"
        };

        // 使用 EntryDecision 风险区间 + 杠杆 推导方向感知止损
        let leverage_for_stop = match zone_1h.confidence {
            rust_trading_bot::entry_zone_analyzer::Confidence::High => self.max_leverage,
            rust_trading_bot::entry_zone_analyzer::Confidence::Medium => {
                (self.min_leverage + self.max_leverage) / 2
            }
            rust_trading_bot::entry_zone_analyzer::Confidence::Low => self.min_leverage,
        } as u32;
        let risk_pct = if entry_decision.price <= 0.0 {
            0.005
        } else {
            ((entry_decision.price - entry_decision.stop_loss) / entry_decision.price)
                .abs()
                .max(0.005)
        };
        let direction_aware_stop_loss = if side == "LONG" {
            final_entry_price * (1.0 - risk_pct)
        } else {
            final_entry_price * (1.0 + risk_pct)
        };
        let leverage_f64 = leverage_for_stop.max(1) as f64;
        let liquidation_price = if side == "LONG" {
            final_entry_price * (1.0 - 1.0 / leverage_f64)
        } else {
            final_entry_price * (1.0 + 1.0 / leverage_f64)
        };
        let safe_stop_loss = if side == "LONG" {
            direction_aware_stop_loss.max(liquidation_price * 1.01)
        } else {
            direction_aware_stop_loss.min(liquidation_price * 0.99)
        };
        let final_stop_loss = ai_signal.stop_loss.unwrap_or(safe_stop_loss);
        info!(
            "💡 {} 止损计算: 入场=${:.8}, 风险={:.2}%, 方向止损=${:.8}, 爆仓价=${:.8}, 最终止损=${:.8}",
            symbol,
            final_entry_price,
            risk_pct * 100.0,
            direction_aware_stop_loss,
            liquidation_price,
            final_stop_loss
        );

        let final_confidence = &ai_signal.confidence;

        // 根据AI confidence调整仓位比例
        let ai_position_multiplier = match final_confidence.as_str() {
            "HIGH" => 1.0,    // 30%全额
            "MEDIUM" => 0.67, // 20%
            "LOW" => 0.5,     // 15%
            _ => 1.0,
        };

        info!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("💰 第5步: 执行试探建仓");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // 根据决策动作执行
        match entry_decision.action {
            EntryAction::EnterNow | EntryAction::EnterWithCaution => {
                self.execute_ai_trial_entry(
                    &symbol,
                    &alert,
                    &zone_1h,
                    &entry_decision,
                    &klines,
                    &klines_5m,
                    current_price,
                    final_entry_price,
                    final_stop_loss,
                    final_confidence.as_str(),
                    ai_position_multiplier,
                    normalized_ai_signal.as_str(),
                    ai_signal.take_profit,
                    false,
                )
                .await?;
            }
            EntryAction::WaitForPullback => {
                let ai_trade_signal = matches!(normalized_ai_signal.as_str(), "BUY" | "SELL");
                let ai_high_confidence = ai_signal.confidence.trim().eq_ignore_ascii_case("HIGH");

                if ai_trade_signal && ai_high_confidence {
                    warn!("⚠️  量化建议等待回调,但AI HIGH信心覆盖决策");
                    info!("   量化理由: {}", entry_decision.reason);
                    info!(
                        "   AI信心: {} | 信号: {} | 理由: {}",
                        ai_signal.confidence, ai_signal.signal, ai_signal.reason
                    );

                    self.execute_ai_trial_entry(
                        &symbol,
                        &alert,
                        &zone_1h,
                        &entry_decision,
                        &klines,
                        &klines_5m,
                        current_price,
                        final_entry_price,
                        final_stop_loss,
                        final_confidence.as_str(),
                        ai_position_multiplier,
                        normalized_ai_signal.as_str(),
                        ai_signal.take_profit,
                        true,
                    )
                    .await?;
                } else {
                    info!("\n📌 等待回调到更好价格: ${:.4}", entry_decision.price);
                    info!("   理由: {}", entry_decision.reason);
                    info!("   AI信心不足以覆盖量化决策,暂不执行");

                    // 加入延迟开仓队列 - 等待回调确认
                    let mut pending = self.pending_entries.write().await;
                    if let Some(existing) = pending.get_mut(&symbol) {
                        existing.retry_count += 1;
                        existing.last_analysis_time = Utc::now();
                        existing.reject_reason = format!("等待回调: {}", entry_decision.reason);
                        let retry_count = existing.retry_count;
                        drop(pending);
                        info!(
                            "📝 {} 已在延迟队列中，更新重试次数: {}",
                            symbol, retry_count
                        );
                    } else {
                        pending.insert(
                            symbol.clone(),
                            PendingEntry {
                                symbol: symbol.clone(),
                                first_signal_time: Utc::now(),
                                last_analysis_time: Utc::now(),
                                alert: alert.clone(),
                                reject_reason: format!("等待回调: {}", entry_decision.reason),
                                retry_count: 0,
                                fund_escape_detected_at: None,
                            },
                        );
                        drop(pending);
                        info!("📝 已加入延迟开仓队列: {} (等待回调)", symbol);
                    }
                }
            }
            EntryAction::Skip => {
                info!("\n⏸️  入场条件不佳,跳过本次信号");
                info!("   理由: {}", entry_decision.reason);
            }
        }

        Ok(())
    }

    /// 统一的试探建仓执行逻辑，便于被不同入口共享
    async fn execute_ai_trial_entry(
        &self,
        symbol: &str,
        alert: &FundAlert,
        zone_1h: &EntryZone,
        entry_decision: &EntryDecision,
        klines: &[Kline],
        klines_5m: &[Kline],
        current_price: f64,
        final_entry_price: f64,
        final_stop_loss: f64,
        final_confidence: &str,
        ai_position_multiplier: f64,
        ai_signal_side: &str,
        take_profit: Option<f64>,
        is_ai_override: bool,
    ) -> Result<()> {
        info!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("💰 第4步: 执行试探建仓");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // ✅ 使用AI判断的方向(BUY/SELL),不再强制根据资金信号决定
        let side = if ai_signal_side.eq_ignore_ascii_case("SELL") {
            "SHORT"
        } else {
            "LONG"
        };
        let mut stop_loss_order_id: Option<String> = None;
        let mut take_profit_order_id: Option<String> = None;

        // 动态计算杠杆和仓位
        let (position_usdt, leverage) = match zone_1h.confidence {
            rust_trading_bot::entry_zone_analyzer::Confidence::High => {
                (self.max_position_usdt, self.max_leverage)
            }
            rust_trading_bot::entry_zone_analyzer::Confidence::Medium => {
                let mid_position = (self.min_position_usdt + self.max_position_usdt) / 2.0;
                let mid_leverage = (self.min_leverage + self.max_leverage) / 2;
                (mid_position, mid_leverage)
            }
            rust_trading_bot::entry_zone_analyzer::Confidence::Low => {
                (self.min_position_usdt, self.min_leverage)
            }
        };

        // 计算试探仓位数量 (使用AI微调后的价格和仓位)
        let adjusted_position = entry_decision.position * ai_position_multiplier;
        // 先基于交易规则动态校验名义金额，避免低于交易所门槛
        let rules = self.exchange.get_symbol_trading_rules(&symbol).await?;
        let min_notional = rules.min_notional.unwrap_or(5.0);
        let base_notional = position_usdt * leverage as f64 * adjusted_position;
        let required_notional = if base_notional < min_notional {
            let adjusted = min_notional * 1.05;
            warn!(
                "📊 {} 基础名义金额 {:.2} USDT < 最低要求 {:.2} USDT, 自动提升到 {:.2} USDT",
                symbol, base_notional, min_notional, adjusted
            );
            adjusted
        } else {
            base_notional
        };
        let trial_quantity = required_notional / final_entry_price;

        info!("💰 试探建仓配置:");
        info!(
            "   AI信心度: {} → 仓位调整: {:.0}%",
            final_confidence,
            adjusted_position * 100.0
        );
        info!("   投入USDT: {:.2}", position_usdt);
        info!("   杠杆倍数: {}x", leverage);
        info!("   开仓数量: {:.6} {}", trial_quantity, alert.coin);
        info!("   入场价格: ${:.4} (AI微调)", final_entry_price);
        info!("   止损价格: ${:.4} (AI微调)", final_stop_loss);

        // 【P0-2】入场区验证 - 拒绝追高
        // 使用最新5m K线收盘价作为信号价，避免 alert.price 恒为 0 造成 inf 偏离
        let signal_price = klines_5m.last().map(|k| k.close).unwrap_or(current_price);
        let entry_zone = (zone_1h.entry_range.0, zone_1h.entry_range.1);
        let indicators = self.analyzer.calculate_indicators(klines);

        if !self
            .validate_entry_zone(
                signal_price,
                final_entry_price,
                entry_zone,
                &indicators,
                is_ai_override,
            )
            .await?
        {
            warn!("⚠️ 入场区验证失败，跳过建仓");

            // 加入延迟开仓队列 - 当前价格不在入场区
            let symbol_owned = symbol.to_string();
            let mut pending = self.pending_entries.write().await;
            if let Some(existing) = pending.get_mut(symbol) {
                existing.retry_count += 1;
                existing.last_analysis_time = Utc::now();
                existing.reject_reason = "价格不在入场区".to_string();
                let retry_count = existing.retry_count;
                drop(pending);
                info!(
                    "📝 {} 已在延迟队列中，更新重试次数: {}",
                    symbol, retry_count
                );
            } else {
                pending.insert(
                    symbol_owned.clone(),
                    PendingEntry {
                        symbol: symbol_owned,
                        first_signal_time: Utc::now(),
                        last_analysis_time: Utc::now(),
                        alert: alert.clone(),
                        reject_reason: "价格不在入场区".to_string(),
                        retry_count: 0,
                        fund_escape_detected_at: None,
                    },
                );
                drop(pending);
                info!("📝 已加入延迟开仓队列: {} (价格不符)", symbol);
            }

            return Ok(());
        }

        info!("✅ 入场区验证通过，继续执行建仓");

        // 设置杠杆和交易模式
        info!(
            "⚙️  设置交易模式: 杠杆={}x, 保证金=逐仓, 模式=单向",
            leverage
        );
        if let Err(e) = self
            .exchange
            .ensure_trading_modes(symbol, leverage, "ISOLATED", false)
            .await
        {
            error!("❌ 设置交易模式失败: {}", e);
            return Err(e);
        }

        // 限价单入场
        let order_side = if side == "LONG" { "BUY" } else { "SELL" };
        match self
            .exchange
            .limit_order(
                symbol,
                trial_quantity,
                order_side,
                final_entry_price,
                Some(side),
                false,
            )
            .await
        {
            Ok(order_id) => {
                info!("✅ 试探建仓订单已提交: {}", order_id);

                let canceled_orders = match self.cancel_symbol_trigger_orders(symbol).await {
                    Ok(ids) => ids,
                    Err(e) => {
                        warn!("⚠️  清理旧触发单失败: {}", e);
                        Vec::new()
                    }
                };
                info!(
                    "🗑️ 取消旧触发单 {} 个: {:?}",
                    canceled_orders.len(),
                    canceled_orders
                );

                // 设置止损挂单
                match self
                    .exchange
                    .set_stop_loss(symbol, side, trial_quantity, final_stop_loss, None)
                    .await
                {
                    Ok(sl_order_id) => {
                        info!(
                            "✅ 止损单已设置 @ ${:.4}, 订单ID: {}",
                            final_stop_loss, sl_order_id
                        );
                        stop_loss_order_id = Some(sl_order_id);
                    }
                    Err(e) => {
                        warn!("⚠️  止损单设置失败: {}", e);
                    }
                }

                // 设置止盈挂单(如果AI提供了take_profit)
                if let Some(take_profit_price) = take_profit {
                    match self
                        .exchange
                        .set_limit_take_profit(symbol, side, trial_quantity, take_profit_price)
                        .await
                    {
                        Ok(tp_order_id) => {
                            info!(
                                "✅ 止盈单已设置 @ ${:.4}, 订单ID: {}",
                                take_profit_price, tp_order_id
                            );
                            take_profit_order_id = Some(tp_order_id);
                        }
                        Err(e) => {
                            warn!("⚠️  止盈单设置失败: {}", e);
                        }
                    }
                } else {
                    info!("ℹ️  AI未提供止盈价,不设置止盈挂单");
                }

                // 成功开仓，从延迟队列移除
                {
                    let mut pending = self.pending_entries.write().await;
                    if pending.remove(symbol).is_some() {
                        info!("✅ {} 成功开仓，已从延迟队列移除", symbol);
                    }
                }

                // 创建试探持仓记录 (使用AI微调后的entry_decision)
                let mut adjusted_entry_decision = entry_decision.clone();
                adjusted_entry_decision.price = final_entry_price;
                adjusted_entry_decision.stop_loss = final_stop_loss;
                adjusted_entry_decision.position = adjusted_position;

                let mut staged_manager = self.staged_manager.write().await;
                match staged_manager.create_trial_position(
                    symbol.to_string(),
                    side.to_string(),
                    &adjusted_entry_decision,
                    position_usdt,
                    leverage as f64,
                ) {
                    Ok(_) => {
                        info!("✅ 试探持仓已记录,等待启动信号补仓70%");

                        // ✅ 新增: 同时记录到 position_trackers，让AI能监控平仓
                        let mut trackers = self.position_trackers.write().await;
                        trackers.insert(
                            symbol.to_string(),
                            PositionTracker {
                                symbol: symbol.to_string(),
                                entry_price: final_entry_price,
                                quantity: trial_quantity,
                                leverage: leverage,
                                side: side.to_string(),
                                stop_loss_order_id: stop_loss_order_id.clone(),
                                take_profit_order_id: take_profit_order_id.clone(),
                                entry_time: Utc::now(),
                                last_check_time: Utc::now(),
                            },
                        );
                        info!("✅ 持仓已同步到AI监控系统 (双轨记录)");
                        drop(trackers);

                        // 记录信号历史
                        let signal_record = SignalRecord {
                            timestamp: Utc::now().to_rfc3339(),
                            signal: if side == "LONG" {
                                "BUY".to_string()
                            } else {
                                "SELL".to_string()
                            },
                            confidence: "MEDIUM".to_string(),
                            reason: format!("试探建仓: {}", entry_decision.reason.clone()),
                            price: entry_decision.price,
                        };
                        self.signal_history.write().await.add(signal_record);
                    }
                    Err(e) => {
                        error!("❌ 创建试探持仓记录失败: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("❌ 试探建仓订单提交失败: {}", e);
            }
        }

        Ok(())
    }

    /// 将 AI 输出的动作统一映射为 BUY/SELL/HOLD/CLOSE，保持 Web 端的信号一致性
    fn normalize_signal_type(raw: &str) -> &'static str {
        let normalized = raw.trim().to_ascii_uppercase();

        match normalized.as_str() {
            "BUY" => "BUY",
            "SELL" => "SELL",
            "HOLD" => "HOLD",
            "CLOSE" => "CLOSE",
            "FULL_CLOSE" | "PARTIAL_CLOSE" => "CLOSE",
            "SET_LIMIT_ORDER" | "SKIP" | "WAIT" | "WAIT_FOR_SIGNAL" => "HOLD",
            value if value.contains("BUY") => "BUY",
            value if value.contains("SELL") => "SELL",
            value if value.contains("CLOSE") => "CLOSE",
            _ => "HOLD",
        }
    }

    /// 将 AI 置信度字符串映射为 0.0-1.0 的数值，统一前端展示口径
    fn map_confidence_to_score(confidence: &str) -> f64 {
        let trimmed = confidence.trim();
        let normalized = trimmed.to_ascii_uppercase();

        match normalized.as_str() {
            "HIGH" => 0.9,
            "MEDIUM" => 0.7,
            "LOW" => 0.5,
            _ => trimmed
                .parse::<f64>()
                .map(|value| value.clamp(0.0, 1.0))
                .unwrap_or(0.0),
        }
    }
}

#[async_trait]
impl SignalContext for IntegratedAITrader {
    fn exchange(&self) -> Arc<BinanceClient> {
        self.exchange.clone()
    }

    fn db(&self) -> &Database {
        &self.db
    }

    fn tracked_coins(&self) -> Arc<RwLock<HashMap<String, FundAlert>>> {
        self.tracked_coins.clone()
    }

    fn coin_ttl_hours(&self) -> i64 {
        self.coin_ttl_hours
    }

    fn max_tracked_coins(&self) -> usize {
        self.max_tracked_coins
    }

    async fn analyze_and_trade(&self, alert: FundAlert) -> Result<()> {
        IntegratedAITrader::analyze_and_trade(self, alert).await
    }
}
