# 2025-12-13 Integrated AI Trader 重启记录
- apps/rust-trading-bot 的 `start_trader.sh` 期望 `../.env` 或本地 `.env`，但仓库根目录的 `.env` 未被脚本加载，执行时会提示找不到 `.env`；需要预先链接或忽略该告警。
- `./target/release/integrated_ai_trader` 默认不会存在，若缺失需先运行 `cargo build --release --bin integrated_ai_trader` 再启动，否则 nohup 会因找不到可执行文件而失败。
- 成功启动后可通过 `curl http://localhost:8080/api/status` 验证 Web API 服务状态。