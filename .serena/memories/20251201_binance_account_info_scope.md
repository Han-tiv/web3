# Binance AccountInfo 范围
- 2025-12-01：ExchangeClient::get_account_info 仅查询 Binance FAPI 合约账户，删除现货和资金钱包累加。
- 返回字段直接来自 futures_account 的 totalMarginBalance / availableBalance / totalUnrealizedProfit。
- 影响：账户权益展示只反映合约保证金，若需现货或资金余额必须单独调用 API。