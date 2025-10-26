pub mod binance_client;
pub mod copy_trader;
pub mod health_monitor;
pub mod telegram_bot;
pub mod telegram_notifier;
pub mod trading_lock;

// 交易所客户端模块
pub mod exchange_trait;
pub mod bitget_client;
pub mod bybit_client;
pub mod gate_client;
pub mod okx_client;
pub mod hyperliquid_client;

// 区块链钱包模块
pub mod bsc_wallet;
pub mod solana_wallet;

// 价格服务
pub mod price_service;

// 多交易所执行器
pub mod multi_exchange_executor;
