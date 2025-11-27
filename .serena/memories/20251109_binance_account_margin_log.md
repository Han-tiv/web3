## 2025-11-09
- `AccountInformation` 结构体现在包含 `totalMarginBalance` 字段，顺序为 totalWalletBalance → totalMarginBalance → availableBalance → totalUnrealizedProfit。
- `BinanceClient::get_account_info` 日志输出改为使用 `totalMarginBalance` 作为合约余额，提示文案为“合约余额”。