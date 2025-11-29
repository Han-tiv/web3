use axum::{
    extract::{Path, Query, State},
    http::{header, Method, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use chrono::{Duration, LocalResult, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tower_http::cors::{Any, CorsLayer};

use crate::database::{AiAnalysisRecord, Database, TradeRecord as DbTradeRecord};
use crate::exchange_trait::{AccountInfo as ExchangeAccountInfo, ExchangeClient};

// ==================== 数据结构 ====================

#[derive(Debug, Clone, Serialize)]
pub struct AccountSummary {
    pub total_equity: f64,
    pub available_balance: f64,
    pub unrealized_pnl: f64,
    pub initial_balance: f64,
    pub total_trades: usize,
    pub win_rate: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct EquityPoint {
    pub timestamp: String,
    pub total_equity: f64,
    pub pnl: f64,
    pub pnl_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Position {
    pub symbol: String,
    pub side: String,
    pub entry_price: f64,
    pub current_price: f64,
    pub quantity: f64,
    pub pnl: f64,
    pub pnl_pct: f64,
    pub entry_time: String,
    pub leverage: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct TradeRecord {
    pub id: String,
    pub symbol: String,
    pub side: String,
    pub entry_price: f64,
    pub exit_price: f64,
    pub quantity: f64,
    pub pnl: f64,
    pub pnl_pct: f64,
    pub entry_time: String,
    pub exit_time: String,
    pub hold_duration: i64, // 秒
}

#[derive(Deserialize)]
pub struct TradesQuery {
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    50
}

// ==================== 共享状态 ====================

pub struct AppState {
    pub db: Database,
    pub exchange: Arc<dyn ExchangeClient>,
    pub initial_balance: f64,
    pub start_time: Instant,
}

impl AppState {
    pub fn new(initial_balance: f64, db: Database, exchange: Arc<dyn ExchangeClient>) -> Self {
        Self {
            db,
            exchange,
            initial_balance,
            start_time: Instant::now(),
        }
    }
}

// ==================== API处理函数 ====================

async fn get_account(State(state): State<Arc<AppState>>) -> Json<AccountSummary> {
    // 从交易所实时获取账户资产
    let account_info = match state.exchange.get_account_info().await {
        Ok(info) => info,
        Err(err) => {
            log::warn!("⚠️  获取实时账户信息失败，使用初始余额回退: {}", err);
            ExchangeAccountInfo {
                total_balance: state.initial_balance,
                available_balance: state.initial_balance,
                unrealized_pnl: 0.0,
                margin_used: 0.0,
            }
        }
    };

    let trades = state.db.list_trades(1000).unwrap_or_default();
    let total_trades = trades.len();
    let win_rate = if total_trades > 0 {
        let winning = trades.iter().filter(|t| t.pnl > 0.0).count();
        winning as f64 / total_trades as f64
    } else {
        0.0
    };

    Json(AccountSummary {
        total_equity: account_info.total_balance,
        available_balance: account_info.available_balance,
        unrealized_pnl: account_info.unrealized_pnl,
        initial_balance: state.initial_balance,
        total_trades,
        win_rate,
    })
}

async fn get_equity_history(State(state): State<Arc<AppState>>) -> Json<Vec<EquityPoint>> {
    let account_info = match state.exchange.get_account_info().await {
        Ok(info) => info,
        Err(err) => {
            log::warn!("⚠️  获取实时权益失败，使用模拟曲线: {}", err);
            ExchangeAccountInfo {
                total_balance: state.initial_balance,
                available_balance: state.initial_balance,
                unrealized_pnl: 0.0,
                margin_used: 0.0,
            }
        }
    };

    let current_equity = account_info.total_balance;
    let pnl = current_equity - state.initial_balance;
    let now = Utc::now();
    const STEPS: usize = 12;
    const INTERVAL_MINUTES: i64 = 15; // 改为15分钟间隔,总跨度3小时
    let denominator = if STEPS > 1 { (STEPS - 1) as f64 } else { 1.0 };

    let mut points = Vec::with_capacity(STEPS);
    for idx in 0..STEPS {
        let progress = if STEPS > 1 {
            idx as f64 / denominator
        } else {
            1.0
        };

        let timestamp =
            (now - Duration::minutes(((STEPS - idx - 1) as i64) * INTERVAL_MINUTES)).to_rfc3339();
        let total_equity = state.initial_balance + pnl * progress;
        let point_pnl = total_equity - state.initial_balance;
        let point_pct = if state.initial_balance.abs() > f64::EPSILON {
            (point_pnl / state.initial_balance) * 100.0
        } else {
            0.0
        };

        points.push(EquityPoint {
            timestamp,
            total_equity,
            pnl: point_pnl,
            pnl_pct: point_pct,
        });
    }

    Json(points)
}

async fn get_positions(State(state): State<Arc<AppState>>) -> Json<Vec<Position>> {
    let exchange_positions = match state.exchange.get_positions().await {
        Ok(list) => list,
        Err(err) => {
            log::warn!("⚠️  获取实时持仓失败: {}", err);
            Vec::new()
        }
    };

    let snapshot_time = Utc::now().to_rfc3339();
    let positions = exchange_positions
        .into_iter()
        .map(|p| {
            let pnl_pct = if p.margin.abs() > f64::EPSILON {
                (p.pnl / p.margin) * 100.0
            } else {
                let notional = p.entry_price * p.size;
                if notional.abs() > f64::EPSILON {
                    (p.pnl / notional) * 100.0
                } else {
                    0.0
                }
            };

            Position {
                symbol: p.symbol,
                side: p.side,
                entry_price: p.entry_price,
                current_price: p.mark_price,
                quantity: p.size,
                pnl: p.pnl,
                pnl_pct,
                entry_time: snapshot_time.clone(),
                leverage: p.leverage.max(0) as u32,
            }
        })
        .collect();

    Json(positions)
}

async fn get_trades(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TradesQuery>,
) -> Json<Vec<TradeRecord>> {
    let limit = query.limit.min(200);
    let records = state.db.list_trades(limit).unwrap_or_default();
    let trades = records
        .into_iter()
        .map(|r| TradeRecord {
            id: r.id.map(|id| id.to_string()).unwrap_or_default(),
            symbol: r.symbol,
            side: r.side,
            entry_price: r.entry_price,
            exit_price: r.exit_price,
            quantity: r.quantity,
            pnl: r.pnl,
            pnl_pct: r.pnl_pct,
            entry_time: r.entry_time,
            exit_time: r.exit_time,
            hold_duration: r.hold_duration,
        })
        .collect();
    Json(trades)
}

#[derive(Debug, Serialize)]
struct SystemStatus {
    online: bool,
    uptime_seconds: u64,
    last_update: String,
    positions_count: usize,
    trades_count: usize,
    ai_analysis_count: usize,
}

async fn get_status(State(state): State<Arc<AppState>>) -> Json<SystemStatus> {
    let uptime = state.start_time.elapsed().as_secs();
    let positions_count = match state.exchange.get_positions().await {
        Ok(list) => list.len(),
        Err(err) => {
            log::warn!("⚠️  获取实时持仓数量失败: {}", err);
            0
        }
    };
    let trades_count = state.db.count_trades().unwrap_or(0);
    let ai_analysis = state.db.list_ai_analysis(None, 1).unwrap_or_default();

    Json(SystemStatus {
        online: true, // 能响应请求就表示在线
        uptime_seconds: uptime,
        last_update: Utc::now().to_rfc3339(),
        positions_count,
        trades_count,
        ai_analysis_count: ai_analysis.len(),
    })
}

async fn get_ai_history(State(state): State<Arc<AppState>>) -> Json<Vec<AiAnalysisRecord>> {
    let records = state.db.list_ai_analysis(None, 100).unwrap_or_default();
    Json(records)
}

async fn get_telegram_signals(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<crate::database::TelegramSignalRecord>> {
    let signals = state.db.list_telegram_signals(50).unwrap_or_default();
    Json(signals)
}

async fn close_position(
    State(_state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
) -> impl IntoResponse {
    // TODO: 实现实际的平仓逻辑
    log::warn!("收到平仓请求: {}", symbol);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": format!("平仓请求已接收: {}", symbol)
        })),
    )
}

async fn health_check() -> &'static str {
    "OK"
}

// ==================== Python信号接收 ====================

/// 原始Telegram消息Payload (Python透传)
#[derive(Debug, Deserialize, Serialize)]
pub struct RawTelegramPayload {
    pub raw_message: String,
    pub timestamp: f64,
    pub source: String, // "telegram_raw"
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramSignalPayload {
    pub symbol: String,
    pub raw_message: String,
    pub timestamp: f64,
}

/// 接收Python监控发来的交易信号
async fn handle_telegram_signal(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TelegramSignalPayload>,
) -> impl IntoResponse {
    let TelegramSignalPayload {
        symbol,
        raw_message,
        timestamp,
    } = payload;

    log::info!("📨 收到Telegram信号: {}", symbol);
    let preview: String = raw_message.chars().take(120).collect();
    log::debug!("   消息预览: {}", preview.replace('\n', " "));

    if let Err(e) = save_telegram_signal(&state.db, &symbol, &raw_message, timestamp) {
        log::error!("❌ 保存信号到数据库失败: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": "error",
                "message": format!("数据库保存失败: {}", e)
            })),
        );
    }

    log::info!("✅ 信号已保存到数据库,等待交易引擎处理");

    // 返回成功响应
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "received",
            "symbol": symbol,
            "queued_at": chrono::Utc::now().to_rfc3339(),
            "message": format!("信号已接收并保存: {}", symbol)
        })),
    )
}

/// 统一封装信号入库逻辑，确保仅依赖最基本的字段。
fn save_telegram_signal(
    db: &Database,
    symbol: &str,
    raw_message: &str,
    timestamp: f64,
) -> crate::database::DbResult<i64> {
    let timestamp_str = format_signal_timestamp(timestamp);
    db.insert_telegram_signal(symbol, raw_message, &timestamp_str)
}

/// Telegram透传的时间戳是秒级浮点数，转为RFC3339便于后续检索与显示。
fn format_signal_timestamp(timestamp: f64) -> String {
    let secs = timestamp.round() as i64;
    match Utc.timestamp_opt(secs, 0) {
        LocalResult::Single(dt) => dt.to_rfc3339(),
        _ => Utc::now().to_rfc3339(),
    }
}

/// 接收Python监控发来的原始Telegram消息
async fn receive_raw_telegram_message(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RawTelegramPayload>,
) -> impl IntoResponse {
    log::info!(
        "📨 收到原始Telegram消息: {} 字节 | 来源: {}",
        payload.raw_message.len(),
        payload.source
    );
    log::debug!(
        "   消息预览: {}...",
        &payload.raw_message[..payload.raw_message.len().min(100)]
    );

    // 解析Valuescan消息格式,提取币种信息用于数据库存储
    // 格式: 💰 【资金异动】$SOL\n现价: $188.83\n24H: +1.62%
    let symbol = extract_symbol_from_message(&payload.raw_message);

    if symbol.is_empty() {
        log::warn!("⚠️  无法从消息中提取币种,跳过存储");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": "error",
                "message": "消息格式不正确: 无法提取币种"
            })),
        );
    }

    // 直接插入到telegram_signals表,让轮询线程异步处理
    // 使用默认评分和类型,Rust后续会重新解析
    let save_result = state.db.insert_telegram_signal(
        &symbol,
        &payload.raw_message,
        &chrono::Utc::now().to_rfc3339(),
    );

    if let Err(e) = save_result {
        log::error!("❌ 保存原始消息到数据库失败: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": "error",
                "message": format!("数据库保存失败: {}", e)
            })),
        );
    }

    log::info!("✅ 原始消息已保存到数据库,等待Rust轮询线程处理: {}", symbol);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "received",
            "symbol": symbol,
            "queued_at": chrono::Utc::now().to_rfc3339(),
            "message": format!("原始消息已接收并排队处理: {}", symbol)
        })),
    )
}

/// 从Valuescan原始消息中提取币种代码
/// 格式: 💰 【资金异动】$SOL 或 【Alpha】$BTC
fn extract_symbol_from_message(text: &str) -> String {
    // 使用简单正则提取 $SYMBOL 格式
    if let Some(caps) = regex::Regex::new(r"\$([A-Z0-9]+)")
        .ok()
        .and_then(|re| re.captures(text))
    {
        if let Some(coin) = caps.get(1) {
            return format!("{}USDT", coin.as_str());
        }
    }
    String::new()
}

// ==================== 路由配置 ====================

fn create_router(state: Arc<AppState>) -> Router {
    // CORS配置
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    Router::new()
        .route("/api/account", get(get_account))
        .route("/api/equity-history", get(get_equity_history))
        .route("/api/positions", get(get_positions))
        .route("/api/trades", get(get_trades))
        .route("/api/status", get(get_status))
        .route("/api/ai-history", get(get_ai_history))
        .route("/api/telegram-signals", get(get_telegram_signals))
        .route("/api/signals", post(handle_telegram_signal)) // 新增: 接收Python信号
        .route("/api/telegram/raw", post(receive_raw_telegram_message)) // 新增: 接收Python原始消息
        .route("/api/positions/:symbol/close", post(close_position))
        .route("/health", get(health_check))
        .layer(cors)
        .with_state(state)
}

// ==================== 启动Web服务器 ====================

pub async fn start_web_server(
    port: u16,
    state: Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_router(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    log::info!("🌐 Web API服务器启动: http://localhost:{}", port);
    log::info!("   - 账户信息: http://localhost:{}/api/account", port);
    log::info!(
        "   - 权益历史: http://localhost:{}/api/equity-history",
        port
    );
    log::info!("   - 当前持仓: http://localhost:{}/api/positions", port);
    log::info!("   - 交易历史: http://localhost:{}/api/trades", port);
    log::info!("   - 系统状态: http://localhost:{}/api/status", port);
    log::info!("   - AI分析历史: http://localhost:{}/api/ai-history", port);

    axum::serve(listener, app).await?;

    Ok(())
}

// ==================== 辅助函数 ====================

impl AppState {
    /// 添加交易记录到数据库
    pub fn add_trade(&self, trade: &TradeRecord) -> Result<(), Box<dyn std::error::Error>> {
        let record = DbTradeRecord {
            id: None,
            symbol: trade.symbol.clone(),
            side: trade.side.clone(),
            entry_price: trade.entry_price,
            exit_price: trade.exit_price,
            quantity: trade.quantity,
            pnl: trade.pnl,
            pnl_pct: trade.pnl_pct,
            entry_time: trade.entry_time.clone(),
            exit_time: trade.exit_time.clone(),
            hold_duration: trade.hold_duration,
            strategy_tag: None,
            notes: None,
            created_at: Some(Utc::now().to_rfc3339()),
        };

        self.db.insert_trade(&record)?;
        Ok(())
    }

    /// 记录AI分析
    pub fn record_ai_analysis(
        &self,
        symbol: &str,
        decision: &str,
        confidence: f64,
        signal_type: Option<String>,
        reason: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let record = AiAnalysisRecord {
            id: None,
            timestamp: Utc::now().to_rfc3339(),
            symbol: symbol.to_string(),
            decision: decision.to_string(),
            confidence,
            signal_type,
            reason: reason.to_string(),
            valuescan_score: None,
            risk_reward_ratio: None,
            entry_price: None,
            stop_loss: None,
            resistance: None,
            support: None,
        };

        self.db.insert_ai_analysis(&record)?;
        Ok(())
    }
}
