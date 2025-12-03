use super::ai::{
    AIDecider, ContextBuilder, DecisionHandler, EntryAnalyzer, KlineFetcher, PositionEvaluator,
};
use super::core::EntryManager;
use super::execution::{
    ActionExecutor, BatchEvaluator, PositionProtector, StagedStopLossMonitor, TrialPositionMonitor,
};
/// 集成AI交易系统 - 整合主力资金监控 + DeepSeek AI + 多交易所执行
///
/// 功能：
/// 1. 监控Telegram主力资金频道(Valuescan 2254462672)
/// 2. 筛选Alpha/FOMO高潜力币种
/// 3. 获取技术数据（K线、指标、关键位）
/// 4. DeepSeek AI综合分析决策
/// 5. 多交易所并发执行
/// 6. 严格风控管理
use super::modules::{config::*, types::*};
use super::utils::converters::{
    map_confidence_to_score, normalize_signal_type, timestamp_ms_to_datetime,
};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use log::{debug, error, info, warn};
use std::collections::{HashMap, HashSet};
use std::env;
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant};
use teloxide::{prelude::Requester, Bot};
use tokio::{
    io::AsyncWriteExt,
    sync::{Mutex, RwLock},
};

use rust_trading_bot::ai::PromptBuilder;
use rust_trading_bot::database::{AiAnalysisRecord, Database, TradeRecord as DbTradeRecord};
use rust_trading_bot::{
    binance_client::BinanceClient,
    deepseek_client::{
        DeepSeekClient, EnhancedPositionAnalysis, Kline, TechnicalIndicators, TradingSignal,
    },
    entry_zone_analyzer::{EntryAction, EntryZoneAnalyzer},
    exchange_trait::{ExchangeClient, Position},
    gemini_client::GeminiClient,
    key_level_finder::KeyLevelFinder,
    launch_signal_detector::LaunchSignalDetector,
    // prompt_templates 已拆分到各 AI client 的 prompts 子模块
    signals::{AlertType, FundAlert, MessageParser, SignalContext},
    staged_position_manager::{StagedPosition, StagedPositionManager},
    technical_analysis::TechnicalAnalyzer,
    trading::OrderManager,
};

#[path = "trader_entry_executor.rs"]
mod trader_entry_executor;

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
    pub entry_manager: Arc<EntryManager>,

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
    pub position_evaluator: PositionEvaluator,
    // AI决策组件
    pub kline_fetcher: Arc<KlineFetcher>,
    pub entry_analyzer: Arc<EntryAnalyzer>,
    pub ai_decider: Arc<AIDecider>,
    // 动作执行组件
    pub action_executor: Arc<ActionExecutor>,
    // 新增监控组件
    pub trial_monitor: Arc<TrialPositionMonitor>,
    pub stop_loss_monitor: Arc<StagedStopLossMonitor>,
    pub position_protector: Arc<PositionProtector>,
    pub batch_evaluator: Arc<BatchEvaluator>,
}

#[derive(Clone)]
#[allow(dead_code)]
pub(crate) struct TrackerSnapshot {
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
    pub entry_price: f64,
    pub entry_time: DateTime<Utc>,
    pub leverage: u32,
    pub stop_loss_order_id: Option<String>,
    pub take_profit_order_id: Option<String>,
}

impl IntegratedAITrader {
    pub async fn new(
        exchange: BinanceClient,
        deepseek_api_key: String,
        gemini_api_key: String,
        db: Database,
    ) -> Result<Arc<Self>> {
        let exchange = Arc::new(exchange);
        let trading_config = TradingConfig::default();
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
        let order_manager = OrderManager::new(exchange.clone());
        let gemini = Arc::new(GeminiClient::new(gemini_api_key));
        let analyzer = Arc::new(TechnicalAnalyzer::new());
        let context_builder = ContextBuilder::new(exchange.clone(), analyzer.clone());
        let db_shared = Arc::new(db.clone());
        let decision_handler = DecisionHandler::new(db_shared);
        let position_evaluator =
            PositionEvaluator::new(gemini.clone(), context_builder, decision_handler);

        let deepseek_client = Arc::new(DeepSeekClient::new(deepseek_api_key));
        let level_finder = Arc::new(KeyLevelFinder::new());
        let entry_zone_analyzer = Arc::new(EntryZoneAnalyzer::default());
        let launch_detector = Arc::new(LaunchSignalDetector::default());
        let staged_manager = Arc::new(RwLock::new(StagedPositionManager::default()));
        let tracked_coins = Arc::new(RwLock::new(HashMap::new()));
        let position_trackers = Arc::new(RwLock::new(HashMap::new()));
        let signal_history = Arc::new(RwLock::new(SignalHistory::new(30)));
        let last_analysis_time = Arc::new(RwLock::new(HashMap::new()));
        let volatility_cache = Arc::new(RwLock::new(HashMap::new()));
        let active_trigger_orders = Arc::new(Mutex::new(Vec::new()));
        let pending_entries = Arc::new(RwLock::new(HashMap::new()));
        let exchange_client: Arc<dyn ExchangeClient + Send + Sync> = exchange.clone();
        let kline_fetcher = Arc::new(KlineFetcher::new(exchange_client));
        let entry_analyzer_component = Arc::new(EntryAnalyzer::new(entry_zone_analyzer.clone()));
        let ai_decider_component =
            Arc::new(AIDecider::new(deepseek_client.clone(), gemini.clone()));
        let risk_limits = RiskLimitConfig {
            max_position_usdt: trading_config.max_position_usdt,
            min_position_usdt: trading_config.min_position_usdt,
            max_leverage: trading_config.max_leverage,
            min_leverage: trading_config.min_leverage,
        };
        let entry_manager_config = EntryManagerConfig {
            exchange: exchange.clone(),
            deepseek: deepseek_client.clone(),
            gemini: gemini.clone(),
            analyzer: analyzer.clone(),
            entry_zone_analyzer: entry_zone_analyzer.clone(),
            staged_manager: staged_manager.clone(),
            position_trackers: position_trackers.clone(),
            pending_entries: pending_entries.clone(),
            signal_history: signal_history.clone(),
            last_analysis_time: last_analysis_time.clone(),
            risk_limits,
            db: db.clone(),
        };
        let entry_manager = Arc::new(EntryManager::new(entry_manager_config));

        let trader = Arc::new_cyclic(move |weak_self| {
            let trial_monitor = Arc::new(TrialPositionMonitor::new(weak_self.clone()));
            let stop_loss_monitor = Arc::new(StagedStopLossMonitor::new(weak_self.clone()));
            let position_protector = Arc::new(PositionProtector::new(weak_self.clone()));
            let batch_evaluator = Arc::new(BatchEvaluator::new(weak_self.clone()));
            let action_executor = Arc::new(ActionExecutor::new(weak_self.clone()));

            Self {
                order_manager,
                exchange,
                deepseek: deepseek_client,
                gemini,
                analyzer,
                level_finder,

                // 初始化新策略模块
                entry_zone_analyzer,
                launch_detector,
                staged_manager,
                entry_manager: entry_manager.clone(),

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

                min_position_usdt: trading_config.min_position_usdt,
                max_position_usdt: trading_config.max_position_usdt,
                min_leverage: trading_config.min_leverage,
                max_leverage: trading_config.max_leverage,

                max_tracked_coins: 100,
                coin_ttl_hours: 24,

                tracked_coins,
                position_trackers,
                signal_history,
                last_analysis_time,
                volatility_cache,
                active_trigger_orders,
                pending_entries,
                db,
                telegram_bot,
                position_evaluator,
                kline_fetcher,
                entry_analyzer: entry_analyzer_component,
                ai_decider: ai_decider_component,
                action_executor,
                trial_monitor,
                stop_loss_monitor,
                position_protector,
                batch_evaluator,
            }
        });

        Ok(trader)
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
    pub(crate) async fn calculate_volatility(&self, symbol: &str) -> Result<f64> {
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
                timestamp: candle.first().copied().unwrap_or_default() as i64,
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
    #[allow(dead_code)]
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

            // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
            // 【第一步】检查试探持仓,检测启动信号并执行补仓
            // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
            if let Err(e) = self.trial_monitor.monitor().await {
                warn!("⚠️ 试探持仓检测失败: {}", e);
            }

            // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
            // 【第二步】检查分批持仓的快速止损
            // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
            if let Err(e) = self.stop_loss_monitor.monitor().await {
                warn!("⚠️ 分批持仓止损检查失败: {}", e);
            }

            // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
            // 【第三步】获取真实持仓并执行小仓位保护
            // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
            let exchange_positions = match self.exchange.get_positions().await {
                Ok(pos) => pos,
                Err(e) => {
                    warn!("⚠️  获取持仓列表失败: {}", e);
                    warn!("🔍 错误详情: {:?}", e);
                    // ✅ Bug Fix: 即使获取失败也使用空vec,不能跳过小仓位保护逻辑
                    Vec::new()
                }
            };

            if let Err(e) = self.position_protector.execute(&exchange_positions).await {
                warn!("⚠️ 小仓位保护失败: {}", e);
            }

            // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
            // 【第四步】构建tracker快照并执行AI批量评估
            // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
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

            if let Err(e) = self
                .batch_evaluator
                .evaluate(tracker_snapshots, &exchange_positions)
                .await
            {
                warn!("⚠️ AI批量评估失败: {}", e);
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
    #[allow(dead_code)] // 预留给未来的自动执行策略
    async fn execute_recommended_actions(
        &self,
        analysis: &EnhancedPositionAnalysis,
        current_symbol: &str,
    ) -> Result<Vec<String>> {
        if analysis.recommended_actions.is_empty() {
            return Ok(Vec::new());
        }

        let mut actions = analysis.recommended_actions.clone();
        actions.sort_by(|a, b| a.priority.cmp(&b.priority));

        let mut results = Vec::with_capacity(actions.len());

        for action in actions {
            let action_type = action.action_type.clone();
            let reason = action.reason.clone();
            let params = action.params;

            match self
                .action_executor
                .execute_single_action(&action_type, params, current_symbol, reason)
                .await
            {
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
    pub(crate) async fn cancel_symbol_trigger_orders(&self, symbol: &str) -> Result<Vec<u64>> {
        self.action_executor
            .cancel_symbol_trigger_orders(symbol)
            .await
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
            let orphaned_minutes = orphaned_duration.num_minutes();
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
                    let entry_time = timestamp_ms_to_datetime(staged.trial_entry_time);
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

        // 【第1步】信号去重 (30s 内只分析一次)
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

        // 【第2步】标准化交易对并补充历史表现
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

        // 【第3步】多周期K线 (5m/15m/1h)
        let (klines_5m, klines_15m, klines_1h) =
            match self.kline_fetcher.fetch_multi_timeframe(&symbol).await {
                Ok(data) => data,
                Err(_) => return Ok(()),
            };

        let current_price = match klines_15m.last() {
            Some(k) => k.close,
            None => return Ok(()),
        };

        // 【第4步】分析入场区
        let (zone_1h, zone_15m, entry_decision) = match self
            .entry_analyzer
            .analyze_entry_zones(&klines_15m, &klines_1h, current_price)
            .await
        {
            Ok(result) => result,
            Err(_) => return Ok(()),
        };

        info!(
            "
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        );
        info!("🤖 第4步: AI综合判断(K线形态优先)");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let alert_type_str = if alert.alert_type == AlertType::FundEscape {
            "资金出逃"
        } else {
            "资金流入"
        };

        // 【第5步】AI综合决策
        let use_valuescan_v2 = *USE_VALUESCAN_V2;
        let (ai_signal, v2_score, v2_risk_reward, v2_resistance, v2_support) = match self
            .ai_decider
            .make_trading_decision(
                &symbol,
                &alert,
                &zone_1h,
                &zone_15m,
                &entry_decision,
                &klines_5m,
                &klines_15m,
                &klines_1h,
                current_price,
                use_valuescan_v2,
            )
            .await
        {
            Ok(result) => result,
            Err(e) => {
                warn!("⚠️ AI评分不足，加入延迟开仓队列: {} => {}", symbol, e);
                let mut pending = self.pending_entries.write().await;
                if let Some(existing) = pending.get_mut(&symbol) {
                    existing.retry_count += 1;
                    existing.last_analysis_time = Utc::now();
                    existing.reject_reason = format!("AI评分不足: {}", e);
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
                            reject_reason: format!("AI评分不足: {}", e),
                            retry_count: 0,
                            fund_escape_detected_at: None,
                        },
                    );
                    drop(pending);
                    info!("📝 已加入延迟开仓队列: {} (AI评分不足)", symbol);
                }
                return Ok(());
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

        // 【第6步】保存AI分析，便于回溯
        let confidence_value = map_confidence_to_score(&ai_signal.confidence);
        let entry_price_value = ai_signal.entry_price.unwrap_or(current_price);
        let stop_loss_value = ai_signal.stop_loss.unwrap_or(entry_decision.stop_loss);
        let decision_text = format!(
            "{} | 入场: ${:.4} | 止损: ${:.4}",
            ai_signal.signal, entry_price_value, stop_loss_value
        );
        let signal_type = normalize_signal_type(&ai_signal.signal);
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

        // 【第7步】根据AI决策执行计划
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
        };
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

        let build_exec_request = |is_ai_override: bool| EntryExecutionRequest {
            symbol: &symbol,
            alert: &alert,
            zone_1h: &zone_1h,
            entry_decision: &entry_decision,
            klines_15m: &klines_15m,
            klines_5m: &klines_5m,
            current_price,
            final_entry_price,
            final_stop_loss,
            final_confidence: final_confidence.as_str(),
            ai_position_multiplier,
            ai_signal_side: normalized_ai_signal.as_str(),
            take_profit: ai_signal.take_profit,
            is_ai_override,
        };

        // 根据决策动作执行
        match entry_decision.action {
            EntryAction::EnterNow | EntryAction::EnterWithCaution => {
                self.execute_ai_trial_entry(build_exec_request(false))
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

                    self.execute_ai_trial_entry(build_exec_request(true))
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

    pub(crate) async fn close_position_fully(&self, symbol: &str) -> Result<()> {
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
    pub(crate) async fn close_position_fully_with_retry(
        &self,
        symbol: &str,
        max_retries: u32,
    ) -> Result<()> {
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
    pub(crate) async fn send_critical_alert(&self, symbol: &str, reason: &str) {
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
    pub(crate) async fn close_position_partially(
        &self,
        symbol: &str,
        close_pct: f64,
    ) -> Result<f64> {
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

fn format_optional_klines(label: &str, data: Option<&[Kline]>) -> String {
    data.map(|klines| PromptBuilder::format_klines(klines, label, 20))
        .unwrap_or_else(|| format!("\n📊 {}周期 K线: 数据不足", label))
}

fn build_entry_prompt(version_label: &str, ctx: &EntryPromptContext<'_>) -> String {
    let change_text = ctx
        .change_24h
        .map(|v| format!("{:+.2}%", v))
        .unwrap_or_else(|| "未知".to_string());
    let signal_label = ctx.signal_type.unwrap_or("未提供");
    let fund_flow_text =
        PromptBuilder::build_fund_flow_text(ctx.alert_type, ctx.fund_type, ctx.alert_message);
    let kline_5m_text = PromptBuilder::format_klines(ctx.klines_5m, "5m", 15);
    let kline_15m_text = PromptBuilder::format_klines(ctx.klines_15m, "15m", 15);
    let kline_1h_text = PromptBuilder::format_klines(ctx.klines_1h, "1h", 20);
    let kline_4h_text = format_optional_klines("4h", ctx.klines_4h);
    let indicator_text = ctx
        .technical_indicators
        .map(PromptBuilder::format_indicators)
        .unwrap_or_else(|| "📊 技术指标: 暂无数据".to_string());
    let key_levels_text = PromptBuilder::identify_key_levels(ctx.klines_1h, ctx.current_price);

    format!(
        r#"【{version_label} 入场分析】
币种: {symbol}
当前价格: ${current_price:.4} | 24h变化: {change_text}
信号类型: {signal_label}
入场动作: {entry_action}
入场理由: {entry_reason}

{fund_flow_text}

🧭 量化参考:
- 1h区域: {zone_1h}
- 15m区域: {zone_15m}

{kline_5m_text}

{kline_15m_text}

{kline_1h_text}

{kline_4h_text}

{key_levels_text}

{indicator_text}
"#,
        version_label = version_label,
        symbol = ctx.symbol,
        current_price = ctx.current_price,
        change_text = change_text,
        signal_label = signal_label,
        entry_action = ctx.entry_action,
        entry_reason = ctx.entry_reason,
        fund_flow_text = fund_flow_text,
        zone_1h = ctx.zone_1h_summary,
        zone_15m = ctx.zone_15m_summary,
        kline_5m_text = kline_5m_text,
        kline_15m_text = kline_15m_text,
        kline_1h_text = kline_1h_text,
        kline_4h_text = kline_4h_text,
        key_levels_text = key_levels_text,
        indicator_text = indicator_text
    )
}

pub fn build_entry_prompt_v2(ctx: &EntryPromptContext<'_>) -> String {
    // 使用 DeepSeek public 方法构建 V2 prompt (包含 JSON 关键词修复)
    // 创建临时 DeepSeekClient 实例调用其 public 方法
    let client = rust_trading_bot::deepseek_client::DeepSeekClient::new(String::new());
    client.build_entry_analysis_prompt_v2(
        ctx.symbol,
        ctx.alert_type,
        ctx.alert_message,
        ctx.flow_text,
        ctx.fund_type,
        ctx.zone_1h_summary,
        ctx.zone_15m_summary,
        ctx.entry_action,
        ctx.entry_reason,
        ctx.klines_5m,
        ctx.klines_15m,
        ctx.klines_1h,
        ctx.current_price,
    )
}

pub fn build_entry_prompt_v1(ctx: &EntryPromptContext<'_>) -> String {
    build_entry_prompt("Valuescan V1", ctx)
}

pub fn build_position_prompt_v2(ctx: &PreparedPositionContext) -> String {
    let kline_5m_text = PromptBuilder::format_klines(&ctx.market.klines_5m, "5m", 15);
    let kline_15m_text = PromptBuilder::format_klines(&ctx.market.klines_15m, "15m", 15);
    let kline_1h_text = PromptBuilder::format_klines(&ctx.market.klines_1h, "1h", 20);
    let indicators_text = PromptBuilder::format_indicators(&ctx.market.indicators);
    let stop_loss_label = ctx.stop_loss_order_id.as_deref().unwrap_or("未设置止损单");
    let take_profit_label = ctx
        .take_profit_order_id
        .as_deref()
        .unwrap_or("未设置止盈单");
    let current_stop_price = ctx
        .current_stop_loss
        .map(|p| format!("${:.4}", p))
        .unwrap_or_else(|| "未设置".to_string());
    let current_tp_price = ctx
        .current_take_profit
        .map(|p| format!("${:.4}", p))
        .unwrap_or_else(|| "未设置".to_string());

    format!(
        r#"【持仓管理 V2】
币种: {symbol} ({side})
当前价格: ${current_price:.4} | 入场价: ${entry_price:.4}
持仓时长: {duration:.2}h | 盈亏: {profit_pct:+.2}%
仓位数量: {quantity:.4} | 最低名义: ${min_notional:.2}

止损单: {stop_loss_label}
止损价格: ${stop_loss_price:.4} | 当前挂单价格: {current_stop_price}
止盈单: {take_profit_label}
止盈价格: {current_tp_price}

支撑阻力分析:
{support_text}

价格偏差:
{deviation_desc}

{kline_5m_text}

{kline_15m_text}

{kline_1h_text}

{indicators_text}
"#,
        symbol = ctx.symbol,
        side = ctx.side,
        current_price = ctx.current_price,
        entry_price = ctx.entry_price,
        duration = ctx.duration,
        profit_pct = ctx.profit_pct,
        quantity = ctx.quantity,
        min_notional = ctx.min_notional,
        stop_loss_label = stop_loss_label,
        stop_loss_price = ctx.stop_loss_price,
        current_stop_price = current_stop_price,
        take_profit_label = take_profit_label,
        current_tp_price = current_tp_price,
        support_text = ctx.support_text,
        deviation_desc = ctx.deviation_desc,
        kline_5m_text = kline_5m_text,
        kline_15m_text = kline_15m_text,
        kline_1h_text = kline_1h_text,
        indicators_text = indicators_text
    )
}
