# /api/status trades_count
- 新增 Database::count_trades() 直接通过 SQL COUNT(*) 获取交易总数，避免为了统计加载整批记录。
- /api/status 的 get_status 端点改为调用 count_trades()，保证 trades_count 精准且不会受 limit 影响。