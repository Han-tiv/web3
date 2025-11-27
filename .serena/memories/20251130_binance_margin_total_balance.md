# Binance AccountInfo 余额口径
- 2025-11-30：ExchangeClient::get_account_info 中 total 由 futures_account.totalWalletBalance 改为 totalMarginBalance。
- 目的：让 AccountInfo.total_balance 等于合约保证金余额，自动包含未实现盈亏。
- 影响：下游 equity/账户展示将直接反映 FAPI totalMarginBalance，如需统计现货或资金账户需看 total/available 的后续累加。