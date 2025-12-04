use std::{
    path::Path,
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::exchange_trait::Position;

/// 统一的数据库结果类型，便于在业务层传播错误。
pub type DbResult<T> = Result<T, DatabaseError>;

/// 数据库错误，包含 SQLite 和互斥锁相关问题。
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("数据库访问失败: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("数据库互斥锁已中毒")]
    Poisoned,
    #[error("记录缺少主键，无法执行更新/删除操作")]
    MissingPrimaryKey,
    #[error("JSON 序列化失败: {0}")]
    Json(#[from] serde_json::Error),
}

/// Phase 2.4 (#12): 记录交易利润参数结构体
pub struct RecordTradeProfitParams<'a> {
    pub symbol: &'a str,
    pub entry_time: i64,
    pub exit_time: i64,
    pub entry_price: f64,
    pub exit_price: f64,
    pub quantity: f64,
    pub side: &'a str,
    pub profit_usdt: f64,
    pub capital_used: f64,
}

/// 核心数据库封装，持有一个 Arc<Mutex<Connection>> 以便在线程间共享。
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// 从文件路径创建数据库，并自动完成 PRAGMA、建表与索引初始化。
    pub fn new<P: AsRef<Path>>(db_path: P) -> DbResult<Self> {
        let conn = Connection::open(db_path)?;
        Self::configure(&conn)?;
        Self::bootstrap(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// 便于测试或内存数据库场景的便捷构造器。
    pub fn from_connection(conn: Connection) -> DbResult<Self> {
        Self::configure(&conn)?;
        Self::bootstrap(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// 暴露底层连接的共享引用，方便组合异步任务。
    pub fn shared_connection(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.conn)
    }

    fn configure(conn: &Connection) -> DbResult<()> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "cache_size", -32_768)?; // 约 32MB
        conn.busy_timeout(Duration::from_secs(5))?;
        Ok(())
    }

    /// 创建表结构与索引，确保数据库可用。
    fn bootstrap(conn: &Connection) -> DbResult<()> {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS trades (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                symbol TEXT NOT NULL,
                side TEXT NOT NULL,
                entry_price REAL NOT NULL,
                exit_price REAL NOT NULL,
                quantity REAL NOT NULL,
                pnl REAL NOT NULL,
                pnl_pct REAL NOT NULL,
                entry_time TEXT NOT NULL,
                exit_time TEXT NOT NULL,
                hold_duration INTEGER NOT NULL,
                strategy_tag TEXT,
                notes TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_trades_symbol ON trades(symbol);
            CREATE INDEX IF NOT EXISTS idx_trades_exit_time ON trades(exit_time);

            CREATE TABLE IF NOT EXISTS ai_analysis (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                symbol TEXT NOT NULL,
                decision TEXT NOT NULL,
                confidence REAL NOT NULL,
                signal_type TEXT,
                reason TEXT NOT NULL,
                valuescan_score REAL,
                risk_reward_ratio REAL,
                entry_price REAL,
                stop_loss REAL,
                resistance REAL,
                support REAL
            );
            CREATE INDEX IF NOT EXISTS idx_ai_symbol_timestamp ON ai_analysis(symbol, timestamp);
            
            CREATE TABLE IF NOT EXISTS pending_tpsl (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                symbol TEXT NOT NULL,
                position_side TEXT NOT NULL,
                quantity REAL NOT NULL,
                take_profit REAL NOT NULL,
                stop_loss REAL NOT NULL,
                created_at INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending'
            );
            CREATE INDEX IF NOT EXISTS idx_pending_tpsl_symbol_side
                ON pending_tpsl(symbol, position_side, status);

            CREATE TABLE IF NOT EXISTS analysis_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                symbol TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                analysis_text TEXT NOT NULL,
                current_price REAL,
                indicators TEXT,
                position TEXT,
                actions TEXT,
                created_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_analysis_logs_symbol_timestamp
                ON analysis_logs(symbol, timestamp);

            CREATE TABLE IF NOT EXISTS trade_profit_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                symbol TEXT NOT NULL,
                entry_time INTEGER NOT NULL,
                exit_time INTEGER NOT NULL,
                entry_price REAL NOT NULL,
                exit_price REAL NOT NULL,
                quantity REAL NOT NULL,
                side TEXT NOT NULL,
                profit_usdt REAL NOT NULL,
                capital_used REAL NOT NULL,
                created_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_profit_history_symbol_time
                ON trade_profit_history(symbol, exit_time);

            CREATE TABLE IF NOT EXISTS telegram_signals (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                symbol TEXT NOT NULL,
                raw_message TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                recommend_action TEXT NOT NULL DEFAULT 'LONG',
                processed INTEGER NOT NULL DEFAULT 0,
                processed_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_telegram_signals_symbol ON telegram_signals(symbol);
            CREATE INDEX IF NOT EXISTS idx_telegram_signals_timestamp ON telegram_signals(timestamp);
        "#,
        )?;
        Self::ensure_column(
            conn,
            "telegram_signals",
            "processed",
            "INTEGER NOT NULL DEFAULT 0",
        )?;
        Self::ensure_column(conn, "telegram_signals", "processed_at", "TEXT")?;
        Ok(())
    }

    fn ensure_column(
        conn: &Connection,
        table: &str,
        column: &str,
        definition: &str,
    ) -> DbResult<()> {
        if Self::has_column(conn, table, column)? {
            return Ok(());
        }
        let sql = format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, definition);
        conn.execute(&sql, [])?;
        Ok(())
    }

    fn has_column(conn: &Connection, table: &str, column: &str) -> DbResult<bool> {
        let pragma = format!("PRAGMA table_info({})", table);
        let mut stmt = conn.prepare(&pragma)?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let name: String = row.get(1)?;
            if name.eq_ignore_ascii_case(column) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn guard(&self) -> DbResult<MutexGuard<'_, Connection>> {
        self.conn.lock().map_err(|_| DatabaseError::Poisoned)
    }

    /// =============== Trades CRUD ===============
    pub fn insert_trade(&self, record: &TradeRecord) -> DbResult<i64> {
        let conn = self.guard()?;
        conn.execute(
            r#"
            INSERT INTO trades (
                symbol, side, entry_price, exit_price, quantity, pnl,
                pnl_pct, entry_time, exit_time, hold_duration, strategy_tag,
                notes, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
        "#,
            params![
                &record.symbol,
                &record.side,
                record.entry_price,
                record.exit_price,
                record.quantity,
                record.pnl,
                record.pnl_pct,
                &record.entry_time,
                &record.exit_time,
                record.hold_duration,
                &record.strategy_tag,
                &record.notes,
                ensure_ts(record.created_at.as_deref().unwrap_or("")),
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_trade(&self, id: i64) -> DbResult<Option<TradeRecord>> {
        let conn = self.guard()?;
        conn.query_row(
            r#"
            SELECT id, symbol, side, entry_price, exit_price, quantity,
                   pnl, pnl_pct, entry_time, exit_time, hold_duration,
                   strategy_tag, notes, created_at
            FROM trades
            WHERE id = ?1
        "#,
            params![id],
            map_trade,
        )
        .optional()
        .map_err(DatabaseError::from)
    }

    pub fn list_trades(&self, limit: usize) -> DbResult<Vec<TradeRecord>> {
        let conn = self.guard()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, symbol, side, entry_price, exit_price, quantity,
                   pnl, pnl_pct, entry_time, exit_time, hold_duration,
                   strategy_tag, notes, created_at
            FROM trades
            ORDER BY exit_time DESC
            LIMIT ?1
        "#,
        )?;
        let rows = stmt.query_map(params![limit as i64], map_trade)?;
        collect_rows(rows)
    }

    /// 统计 trades 表总数，避免为了计数拉取整批记录。
    pub fn count_trades(&self) -> DbResult<usize> {
        let conn = self.guard()?;
        let total: i64 = conn.query_row(
            r#"
            SELECT COUNT(*)
            FROM trades
        "#,
            [],
            |row| row.get(0),
        )?;

        Ok(total.max(0) as usize)
    }

    /// 统计 ai_analysis 表总数，供 AI 分析面板或状态接口使用。
    pub fn count_ai_analysis(&self) -> DbResult<usize> {
        let conn = self.guard()?;
        let total: i64 = conn.query_row(
            r#"
            SELECT COUNT(*)
            FROM ai_analysis
        "#,
            [],
            |row| row.get(0),
        )?;

        Ok(total.max(0) as usize)
    }

    pub fn update_trade(&self, record: &TradeRecord) -> DbResult<()> {
        let id = record.id.ok_or(DatabaseError::MissingPrimaryKey)?;
        let conn = self.guard()?;
        conn.execute(
            r#"
            UPDATE trades
            SET symbol = ?1,
                side = ?2,
                entry_price = ?3,
                exit_price = ?4,
                quantity = ?5,
                pnl = ?6,
                pnl_pct = ?7,
                entry_time = ?8,
                exit_time = ?9,
                hold_duration = ?10,
                strategy_tag = ?11,
                notes = ?12,
                created_at = ?13
            WHERE id = ?14
        "#,
            params![
                &record.symbol,
                &record.side,
                record.entry_price,
                record.exit_price,
                record.quantity,
                record.pnl,
                record.pnl_pct,
                &record.entry_time,
                &record.exit_time,
                record.hold_duration,
                &record.strategy_tag,
                &record.notes,
                ensure_ts(record.created_at.as_deref().unwrap_or("")),
                id
            ],
        )?;
        Ok(())
    }

    pub fn delete_trade(&self, id: i64) -> DbResult<()> {
        let conn = self.guard()?;
        conn.execute("DELETE FROM trades WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// =============== AI Analysis CRUD ===============
    pub fn insert_ai_analysis(&self, record: &AiAnalysisRecord) -> DbResult<i64> {
        let conn = self.guard()?;
        let timestamp = ensure_ts(&record.timestamp);
        let reason = ensure_reason(&record.decision, &record.reason);
        conn.execute(
            r#"
            INSERT INTO ai_analysis (
                timestamp, symbol, decision, confidence, signal_type, reason,
                valuescan_score, risk_reward_ratio, entry_price, stop_loss, resistance, support
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        "#,
            params![
                timestamp,
                &record.symbol,
                &record.decision,
                record.confidence,
                &record.signal_type,
                reason,
                record.valuescan_score,
                record.risk_reward_ratio,
                record.entry_price,
                record.stop_loss,
                record.resistance,
                record.support,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_ai_analysis(
        &self,
        symbol: Option<&str>,
        limit: usize,
    ) -> DbResult<Vec<AiAnalysisRecord>> {
        let conn = self.guard()?;
        if let Some(symbol) = symbol {
            let mut stmt = conn.prepare(
                r#"
                SELECT id, timestamp, symbol, decision, confidence, signal_type, reason
                FROM ai_analysis
                WHERE symbol = ?1
                ORDER BY timestamp DESC
                LIMIT ?2
            "#,
            )?;
            let rows = stmt.query_map(params![symbol, limit as i64], map_ai)?;
            collect_rows(rows)
        } else {
            let mut stmt = conn.prepare(
                r#"
                SELECT id, timestamp, symbol, decision, confidence, signal_type, reason
                FROM ai_analysis
                ORDER BY timestamp DESC
                LIMIT ?1
            "#,
            )?;
            let rows = stmt.query_map(params![limit as i64], map_ai)?;
            collect_rows(rows)
        }
    }

    pub fn update_ai_analysis(&self, record: &AiAnalysisRecord) -> DbResult<()> {
        let id = record.id.ok_or(DatabaseError::MissingPrimaryKey)?;
        let conn = self.guard()?;
        let timestamp = ensure_ts(&record.timestamp);
        let reason = ensure_reason(&record.decision, &record.reason);
        conn.execute(
            r#"
            UPDATE ai_analysis
            SET timestamp = ?1,
                symbol = ?2,
                decision = ?3,
                confidence = ?4,
                signal_type = ?5,
                reason = ?6
            WHERE id = ?7
        "#,
            params![
                timestamp,
                &record.symbol,
                &record.decision,
                record.confidence,
                &record.signal_type,
                reason,
                id
            ],
        )?;
        Ok(())
    }

    pub fn delete_ai_analysis(&self, id: i64) -> DbResult<()> {
        let conn = self.guard()?;
        conn.execute("DELETE FROM ai_analysis WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// =============== Gemini Analysis Logs ===============
    pub fn save_analysis_log(
        &self,
        symbol: &str,
        analysis_text: &str,
        current_price: f64,
        indicators: &serde_json::Value,
        position: Option<&Position>,
        actions: Option<&str>,
    ) -> DbResult<i64> {
        let conn = self.guard()?;
        let timestamp = Utc::now().timestamp();
        let indicators_json = serde_json::to_string(indicators)?;
        let position_json = position.map(serde_json::to_string).transpose()?;

        conn.execute(
            r#"
            INSERT INTO analysis_logs (
                symbol, timestamp, analysis_text, current_price, indicators, position, actions, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
            params![
                symbol,
                timestamp,
                analysis_text,
                current_price,
                indicators_json,
                position_json.as_deref(),
                actions,
                timestamp
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// =============== Trade Profit History ===============
    pub fn record_trade_profit(&self, params: &RecordTradeProfitParams<'_>) -> DbResult<()> {
        let conn = self.guard()?;
        let created_at = Utc::now().timestamp();
        conn.execute(
            r#"
            INSERT INTO trade_profit_history (
                symbol, entry_time, exit_time, entry_price, exit_price,
                quantity, side, profit_usdt, capital_used, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        "#,
            rusqlite::params![
                params.symbol,
                params.entry_time,
                params.exit_time,
                params.entry_price,
                params.exit_price,
                params.quantity,
                params.side,
                params.profit_usdt,
                params.capital_used,
                created_at
            ],
        )?;
        Ok(())
    }

    pub fn get_last_profit(&self, symbol: &str) -> DbResult<Option<f64>> {
        let conn = self.guard()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT profit_usdt
            FROM trade_profit_history
            WHERE symbol = ?1
            ORDER BY exit_time DESC
            LIMIT 1
        "#,
        )?;
        let profit = stmt.query_row([symbol], |row| row.get(0)).optional()?;
        Ok(profit)
    }

    /// =============== Pending TP/SL 队列 ===============
    /// 入队待补设的止盈止损信息。
    pub fn enqueue_pending_tpsl(
        &self,
        symbol: &str,
        position_side: &str,
        quantity: f64,
        take_profit: f64,
        stop_loss: f64,
    ) -> DbResult<i64> {
        let conn = self.guard()?;
        let created_at = Utc::now().timestamp();
        conn.execute(
            r#"
            INSERT INTO pending_tpsl (
                symbol, position_side, quantity, take_profit, stop_loss, created_at, status
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending')
        "#,
            params![
                symbol,
                position_side,
                quantity,
                take_profit,
                stop_loss,
                created_at
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// 获取某个方向上最早的待补设止盈止损记录。
    pub fn next_pending_tpsl(
        &self,
        symbol: &str,
        position_side: &str,
    ) -> DbResult<Option<PendingTpSlRecord>> {
        let conn = self.guard()?;
        conn.query_row(
            r#"
            SELECT id, symbol, position_side, quantity, take_profit, stop_loss, created_at, status
            FROM pending_tpsl
            WHERE symbol = ?1
              AND position_side = ?2
              AND status = 'pending'
            ORDER BY created_at ASC
            LIMIT 1
        "#,
            params![symbol, position_side],
            map_pending_tpsl,
        )
        .optional()
        .map_err(DatabaseError::from)
    }

    /// 更新 pending 记录的状态。
    pub fn update_pending_tpsl_status(&self, id: i64, status: PendingTpSlStatus) -> DbResult<()> {
        let conn = self.guard()?;
        conn.execute(
            r#"
            UPDATE pending_tpsl
            SET status = ?1
            WHERE id = ?2
        "#,
            params![status.as_str(), id],
        )?;
        Ok(())
    }

    /// =============== Telegram Signals CRUD ===============
    /// 保存Telegram信号到数据库
    pub fn insert_telegram_signal(
        &self,
        symbol: &str,
        raw_message: &str,
        timestamp: &str,
    ) -> DbResult<i64> {
        let conn = self.guard()?;
        // 简化插入：只保存必要的3个字段
        conn.execute(
            r#"
            INSERT INTO telegram_signals (
                symbol, raw_message, timestamp, processed
            ) VALUES (?1, ?2, ?3, 0)
        "#,
            params![symbol, raw_message, timestamp],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// 获取最近的Telegram信号 (limit条,按时间倒序)
    pub fn list_telegram_signals(&self, limit: usize) -> DbResult<Vec<TelegramSignalRecord>> {
        let conn = self.guard()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, symbol, raw_message, timestamp, recommend_action, created_at, processed, processed_at
            FROM telegram_signals
            ORDER BY timestamp DESC
            LIMIT ?1
        "#,
        )?;
        let rows = stmt.query_map(params![limit as i64], map_telegram_signal)?;
        collect_rows(rows)
    }

    /// 获取指定币种的Telegram信号
    pub fn list_telegram_signals_by_symbol(
        &self,
        symbol: &str,
        limit: usize,
    ) -> DbResult<Vec<TelegramSignalRecord>> {
        let conn = self.guard()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, symbol, raw_message, timestamp, recommend_action, created_at, processed, processed_at
            FROM telegram_signals
            WHERE symbol = ?1
            ORDER BY timestamp DESC
            LIMIT ?2
        "#,
        )?;
        let rows = stmt.query_map(params![symbol, limit as i64], map_telegram_signal)?;
        collect_rows(rows)
    }

    /// 获取所有未处理的Telegram信号 (时间顺序)
    pub fn list_unprocessed_telegram_signals(
        &self,
        limit: usize,
    ) -> DbResult<Vec<TelegramSignalRecord>> {
        let conn = self.guard()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, symbol, raw_message, timestamp, created_at, processed, processed_at
            FROM telegram_signals
            WHERE processed = 0
            ORDER BY timestamp ASC
            LIMIT ?1
        "#,
        )?;
        let rows = stmt.query_map(params![limit as i64], map_telegram_signal)?;
        collect_rows(rows)
    }

    /// 标记指定信号为已处理
    pub fn mark_telegram_signal_processed(&self, id: i64) -> DbResult<()> {
        let conn = self.guard()?;
        conn.execute(
            r#"
            UPDATE telegram_signals
            SET processed = 1,
                processed_at = datetime('now')
            WHERE id = ?1
        "#,
            params![id],
        )?;
        Ok(())
    }
}

/// =============== 数据记录结构体 ===============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub id: Option<i64>,
    pub symbol: String,
    pub side: String,
    pub entry_price: f64,
    pub exit_price: f64,
    pub quantity: f64,
    pub pnl: f64,
    pub pnl_pct: f64,
    pub entry_time: String,
    pub exit_time: String,
    pub hold_duration: i64,
    pub strategy_tag: Option<String>,
    pub notes: Option<String>,
    /// created_at 可用来覆盖默认时间，若为空字符串则自动生成。
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAnalysisRecord {
    pub id: Option<i64>,
    pub timestamp: String,
    pub symbol: String,
    pub decision: String,
    pub confidence: f64,
    pub signal_type: Option<String>,
    pub reason: String,
    // Valuescan V2 扩展字段
    pub valuescan_score: Option<f64>,   // V2评分 (0-10)
    pub risk_reward_ratio: Option<f64>, // 风险收益比
    pub entry_price: Option<f64>,       // 入场价
    pub stop_loss: Option<f64>,         // 止损价
    pub resistance: Option<f64>,        // 阻力位
    pub support: Option<f64>,           // 支撑位
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTpSlRecord {
    pub id: Option<i64>,
    pub symbol: String,
    pub position_side: String,
    pub quantity: f64,
    pub take_profit: f64,
    pub stop_loss: f64,
    pub created_at: i64,
    pub status: PendingTpSlStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PendingTpSlStatus {
    Pending,
    Completed,
    Failed,
}

impl PendingTpSlStatus {
    fn from_str(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            _ => Self::Pending,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramSignalRecord {
    pub id: Option<i64>,
    pub symbol: String,
    pub raw_message: String,
    pub timestamp: String,
    pub created_at: String,
    pub processed: bool,
    pub processed_at: Option<String>,
}

fn ensure_ts(value: &str) -> String {
    if value.trim().is_empty() {
        Utc::now().to_rfc3339()
    } else {
        value.to_string()
    }
}

fn ensure_reason(decision: &str, reason: &str) -> String {
    if reason.trim().is_empty() {
        decision.to_string()
    } else {
        reason.to_string()
    }
}

fn map_trade(row: &Row<'_>) -> rusqlite::Result<TradeRecord> {
    Ok(TradeRecord {
        id: Some(row.get(0)?),
        symbol: row.get(1)?,
        side: row.get(2)?,
        entry_price: row.get(3)?,
        exit_price: row.get(4)?,
        quantity: row.get(5)?,
        pnl: row.get(6)?,
        pnl_pct: row.get(7)?,
        entry_time: row.get(8)?,
        exit_time: row.get(9)?,
        hold_duration: row.get(10)?,
        strategy_tag: row.get(11)?,
        notes: row.get(12)?,
        created_at: Some(row.get(13)?),
    })
}

fn map_ai(row: &Row<'_>) -> rusqlite::Result<AiAnalysisRecord> {
    Ok(AiAnalysisRecord {
        id: Some(row.get(0)?),
        timestamp: row.get(1)?,
        symbol: row.get(2)?,
        decision: row.get(3)?,
        confidence: row.get(4)?,
        signal_type: row.get(5)?,
        reason: row.get(6)?,
        valuescan_score: row.get(7)?,
        risk_reward_ratio: row.get(8)?,
        entry_price: row.get(9)?,
        stop_loss: row.get(10)?,
        resistance: row.get(11)?,
        support: row.get(12)?,
    })
}

fn map_pending_tpsl(row: &Row<'_>) -> rusqlite::Result<PendingTpSlRecord> {
    let status: String = row.get(7)?;
    Ok(PendingTpSlRecord {
        id: Some(row.get(0)?),
        symbol: row.get(1)?,
        position_side: row.get(2)?,
        quantity: row.get(3)?,
        take_profit: row.get(4)?,
        stop_loss: row.get(5)?,
        created_at: row.get(6)?,
        status: PendingTpSlStatus::from_str(&status),
    })
}

fn map_telegram_signal(row: &Row<'_>) -> rusqlite::Result<TelegramSignalRecord> {
    Ok(TelegramSignalRecord {
        id: Some(row.get(0)?),
        symbol: row.get(1)?,
        raw_message: row.get(2)?,
        timestamp: row.get(3)?,
        created_at: row.get(4)?,
        processed: {
            let value: i64 = row.get(5)?;
            value != 0
        },
        processed_at: row.get::<_, Option<String>>(6)?,
    })
}

fn collect_rows<T, F>(rows: rusqlite::MappedRows<'_, F>) -> DbResult<Vec<T>>
where
    F: FnMut(&Row<'_>) -> rusqlite::Result<T>,
{
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(DatabaseError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn memory_db() -> Database {
        Database::from_connection(Connection::open_in_memory().unwrap()).unwrap()
    }

    #[test]
    fn test_trade_crud_cycle() {
        let db = memory_db();
        let mut trade = TradeRecord {
            id: None,
            symbol: "ETHUSDT".into(),
            side: "SHORT".into(),
            entry_price: 2000.0,
            exit_price: 1900.0,
            quantity: 1.0,
            pnl: 100.0,
            pnl_pct: 5.0,
            entry_time: "2024-01-01T00:00:00Z".into(),
            exit_time: "2024-01-01T01:00:00Z".into(),
            hold_duration: 3600,
            strategy_tag: Some("scalp".into()),
            notes: None,
            created_at: None,
        };
        let id = db.insert_trade(&trade).unwrap();
        trade.id = Some(id);
        trade.notes = Some("updated".into());
        db.update_trade(&trade).unwrap();
        let stored = db.get_trade(id).unwrap().unwrap();
        assert_eq!(stored.notes.as_deref(), Some("updated"));
        db.delete_trade(id).unwrap();
        assert!(db.get_trade(id).unwrap().is_none());
    }

    #[test]
    fn test_pending_tpsl_flow() {
        let db = memory_db();
        let id = db
            .enqueue_pending_tpsl("ETHUSDT", "LONG", 0.01, 2550.0, 2450.0)
            .unwrap();
        let record = db
            .next_pending_tpsl("ETHUSDT", "LONG")
            .unwrap()
            .expect("record exists");
        assert_eq!(record.id, Some(id));
        db.update_pending_tpsl_status(id, PendingTpSlStatus::Completed)
            .unwrap();
        assert!(db.next_pending_tpsl("ETHUSDT", "LONG").unwrap().is_none());
    }
}
