-- Performance Optimization SQL Scripts for Crypto Bot
-- Phase 2: Database Index Optimization
-- Target: 90%+ query performance improvement

-- ==========================================
-- TASK TABLE PERFORMANCE INDEXES
-- ==========================================

-- 1. 核心查询优化：任务列表按优先级和状态排序
-- 使用场景：GetTasksForAutomation(), GetTasksForHuman()
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_priority_status_created
ON tasks (priority DESC, status, created_at DESC)
WHERE status IN ('pending', 'in_progress');

-- 2. 自动化任务查询优化
-- 使用场景：scheduler 每30秒查询自动执行任务
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_automation_ready
ON tasks (auto_executable, status, priority DESC, deadline)
WHERE auto_executable = true AND status = 'pending';

-- 3. 人工任务查询优化
-- 使用场景：人工工作时间(19:00-22:00)查询
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_human_priority
ON tasks (status, auto_executable, priority DESC, estimated_earning DESC)
WHERE status = 'pending' AND (auto_executable = false OR array_length(requires_human, 1) > 0);

-- 4. 时间范围查询优化
-- 使用场景：日统计生成、过期任务处理
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_time_range
ON tasks (created_at, completed_at, deadline)
WHERE created_at IS NOT NULL;

-- 5. 任务类型和来源分析
-- 使用场景：统计分析、效果评估
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_type_source_status
ON tasks (type, source, status, created_at DESC);

-- 6. 收益分析优化
-- 使用场景：收益统计、ROI计算
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_earnings
ON tasks (status, actual_earning, estimated_earning, completed_at)
WHERE status = 'completed' AND actual_earning > 0;

-- ==========================================
-- PROJECT TABLE PERFORMANCE INDEXES
-- ==========================================

-- 7. 活跃项目监控优化
-- 使用场景：项目状态更新、监控列表
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_projects_monitoring
ON projects (monitor_enabled, airdrop_status, last_checked)
WHERE monitor_enabled = true;

-- 8. 项目分类查询优化
-- 使用场景：按类别筛选项目
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_projects_category_status
ON projects (category, airdrop_status, created_at DESC);

-- ==========================================
-- EARNING TABLE PERFORMANCE INDEXES
-- ==========================================

-- 9. 收益时间序列优化
-- 使用场景：日/周/月收益统计
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_earnings_time_series
ON earnings (earned_at DESC, usd_value, token);

-- 10. 任务收益关联优化
-- 使用场景：任务收益分析
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_earnings_task_project
ON earnings (task_id, project_id, earned_at DESC);

-- ==========================================
-- NOTIFICATION TABLE PERFORMANCE INDEXES
-- ==========================================

-- 11. 未读通知查询优化
-- 使用场景：实时通知推送
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_notifications_unread
ON notifications (read, priority DESC, created_at DESC)
WHERE read = false;

-- ==========================================
-- DAILY_STATS TABLE PERFORMANCE INDEXES
-- ==========================================

-- 12. 统计数据时间查询优化
-- 使用场景：历史统计查询
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_daily_stats_date
ON daily_stats (date DESC);

-- ==========================================
-- COMPOSITE INDEXES FOR COMPLEX QUERIES
-- ==========================================

-- 13. 多条件任务搜索优化
-- 使用场景：复杂的任务筛选 (API查询参数)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_complex_filter
ON tasks (status, type, priority, auto_executable, created_at DESC);

-- 14. 项目任务关联查询优化
-- 使用场景：项目详情页面显示相关任务
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_project_status
ON tasks (project_id, status, priority DESC, created_at DESC)
WHERE project_id IS NOT NULL;

-- ==========================================
-- PARTIAL INDEXES FOR MEMORY EFFICIENCY
-- ==========================================

-- 15. 仅索引活跃数据
-- 使用场景：减少索引大小，提升查询速度
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_active_only
ON tasks (priority DESC, created_at DESC)
WHERE status IN ('pending', 'in_progress') AND created_at > NOW() - INTERVAL '30 days';

-- ==========================================
-- JSONB INDEXES FOR METADATA QUERIES
-- ==========================================

-- 16. 元数据查询优化 (如果需要搜索metadata字段)
-- 使用场景：基于元数据的高级搜索
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_metadata_gin
ON tasks USING gin(metadata);

-- ==========================================
-- TEXT SEARCH INDEXES
-- ==========================================

-- 17. 全文搜索优化
-- 使用场景：任务标题和描述搜索
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_fulltext
ON tasks USING gin(to_tsvector('english', title || ' ' || description));

-- ==========================================
-- QUERY PERFORMANCE MONITORING
-- ==========================================

-- 创建查询性能监控视图
CREATE OR REPLACE VIEW v_slow_queries AS
SELECT
    query,
    calls,
    total_time,
    mean_time,
    rows,
    100.0 * shared_blks_hit / NULLIF(shared_blks_hit + shared_blks_read, 0) AS hit_percent
FROM pg_stat_statements
WHERE calls > 100
ORDER BY mean_time DESC
LIMIT 20;

-- ==========================================
-- DATABASE MAINTENANCE COMMANDS
-- ==========================================

-- 更新表统计信息以优化查询计划
ANALYZE tasks;
ANALYZE projects;
ANALYZE earnings;
ANALYZE notifications;
ANALYZE daily_stats;

-- 清理无用的索引统计
REINDEX TABLE tasks;

-- ==========================================
-- PERFORMANCE BENCHMARKS
-- ==========================================

-- 测试查询性能 (运行前后对比)
/*
-- 1. 自动化任务查询 (目标 <10ms)
EXPLAIN (ANALYZE, BUFFERS)
SELECT * FROM tasks
WHERE auto_executable = true AND status = 'pending'
ORDER BY priority DESC, created_at DESC
LIMIT 50;

-- 2. 高优先级人工任务 (目标 <15ms)
EXPLAIN (ANALYZE, BUFFERS)
SELECT * FROM tasks
WHERE status = 'pending' AND (auto_executable = false OR array_length(requires_human, 1) > 0)
ORDER BY priority DESC, estimated_earning DESC
LIMIT 20;

-- 3. 日统计查询 (目标 <50ms)
EXPLAIN (ANALYZE, BUFFERS)
SELECT
    COUNT(*) as total_tasks,
    COUNT(*) FILTER (WHERE status = 'completed') as completed,
    SUM(actual_earning) FILTER (WHERE status = 'completed') as total_earned
FROM tasks
WHERE created_at >= CURRENT_DATE - INTERVAL '7 days';
*/