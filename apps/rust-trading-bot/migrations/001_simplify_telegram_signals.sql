-- 信号表简化迁移
-- 日期: 2025-11-29
-- 目的: 删除recommend_action/score/signal_type字段，简化架构

-- ======================================================================
-- 第1步: 备份现有数据
-- ======================================================================

-- 创建备份表
DROP TABLE IF EXISTS telegram_signals_backup;
CREATE TABLE telegram_signals_backup AS 
SELECT * FROM telegram_signals;

SELECT COUNT(*) as backup_count FROM telegram_signals_backup;

-- ======================================================================
-- 第2步: 创建新表结构
-- ======================================================================

-- 删除旧表
DROP TABLE IF EXISTS telegram_signals;

-- 创建新的简化表
CREATE TABLE telegram_signals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,                  -- 交易对 (例如: BTCUSDT)
    raw_message TEXT NOT NULL,             -- 原始Telegram消息
    timestamp TEXT NOT NULL,               -- ISO 8601格式时间戳
    processed INTEGER NOT NULL DEFAULT 0,  -- 是否已处理 (0=未处理, 1=已处理)
    processed_at TEXT,                     -- 处理时间
    created_at TEXT NOT NULL DEFAULT (datetime('now'))  -- 创建时间
);

-- ======================================================================
-- 第3步: 创建索引
-- ======================================================================

-- 索引1: 查询未处理信号 (最常用)
CREATE INDEX idx_telegram_signals_processed 
ON telegram_signals(processed);

-- 索引2: 按币种查询
CREATE INDEX idx_telegram_signals_symbol 
ON telegram_signals(symbol);

-- 索引3: 按时间查询
CREATE INDEX idx_telegram_signals_timestamp 
ON telegram_signals(timestamp);

-- 索引4: 组合索引 (未处理+时间，用于轮询)
CREATE INDEX idx_telegram_signals_processed_timestamp 
ON telegram_signals(processed, timestamp);

-- ======================================================================
-- 第4步: 迁移历史数据（可选）
-- ======================================================================

-- 从备份表迁移数据到新表（只保留必要字段）
INSERT INTO telegram_signals (
    symbol,
    raw_message,
    timestamp,
    processed,
    processed_at,
    created_at
)
SELECT 
    symbol,
    raw_message,
    timestamp,
    processed,
    processed_at,
    created_at
FROM telegram_signals_backup;

-- ======================================================================
-- 第5步: 验证迁移结果
-- ======================================================================

-- 验证记录数量
SELECT 
    (SELECT COUNT(*) FROM telegram_signals_backup) as old_count,
    (SELECT COUNT(*) FROM telegram_signals) as new_count,
    CASE 
        WHEN (SELECT COUNT(*) FROM telegram_signals_backup) = 
             (SELECT COUNT(*) FROM telegram_signals)
        THEN '✅ 迁移成功'
        ELSE '❌ 记录数不匹配'
    END as status;

-- 查看新表结构
.schema telegram_signals

-- 查看最近5条记录
SELECT 
    id,
    symbol,
    substr(raw_message, 1, 50) as message_preview,
    timestamp,
    processed,
    created_at
FROM telegram_signals
ORDER BY created_at DESC
LIMIT 5;

-- ======================================================================
-- 清理说明
-- ======================================================================

-- 迁移完成后，如果确认无问题，可以删除备份表:
-- DROP TABLE telegram_signals_backup;

-- 但建议保留备份表几天，以防需要回滚
