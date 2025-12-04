// 多交易所并发执行器
use crate::exchanges::traits::*;
use anyhow::Result;
use log::{error, info, warn};
use std::sync::Arc;
use tokio::task::JoinSet;

/// 交易信号类型
#[derive(Debug, Clone)]
pub enum SignalType {
    OpenLong(String),  // 开多
    OpenShort(String), // 开空
    Close(String),     // 平仓
}

/// 多交易所执行器
pub struct MultiExchangeExecutor {
    exchanges: Vec<Arc<dyn ExchangeClient>>,
    leverage: u32,
    margin: f64,
    margin_type: String,
    dual_side_position: bool,
}

impl MultiExchangeExecutor {
    pub fn new(
        exchanges: Vec<Arc<dyn ExchangeClient>>,
        leverage: u32,
        margin: f64,
        margin_type: String,
        dual_side_position: bool,
    ) -> Self {
        Self {
            exchanges,
            leverage,
            margin,
            margin_type,
            dual_side_position,
        }
    }

    /// 并发执行信号到所有交易所
    pub async fn execute_signal(&self, signal: SignalType) -> Vec<Result<String>> {
        let mut tasks = JoinSet::new();

        for exchange in &self.exchanges {
            let exchange_clone = Arc::clone(exchange);
            let signal_clone = signal.clone();
            let leverage = self.leverage;
            let margin = self.margin;
            let margin_type = self.margin_type.clone();
            let dual_side = self.dual_side_position;

            tasks.spawn(async move {
                Self::execute_on_exchange(
                    exchange_clone,
                    signal_clone,
                    leverage,
                    margin,
                    &margin_type,
                    dual_side,
                )
                .await
            });
        }

        let mut results = Vec::new();
        while let Some(result) = tasks.join_next().await {
            match result {
                Ok(exec_result) => results.push(exec_result),
                Err(e) => results.push(Err(anyhow::anyhow!("任务执行失败: {}", e))),
            }
        }

        results
    }

    /// 在单个交易所执行信号
    async fn execute_on_exchange(
        exchange: Arc<dyn ExchangeClient>,
        signal: SignalType,
        leverage: u32,
        margin: f64,
        margin_type: &str,
        dual_side: bool,
    ) -> Result<String> {
        let exchange_name = exchange.get_exchange_name();

        match signal {
            SignalType::OpenLong(symbol) => {
                info!("[{}] 📈 执行开多: {}", exchange_name, symbol);

                // 设置杠杆和保证金模式
                if let Err(e) = exchange.set_leverage(&symbol, leverage).await {
                    warn!("[{}] 设置杠杆失败: {}", exchange_name, e);
                }

                if let Err(e) = exchange.set_margin_type(&symbol, margin_type).await {
                    warn!("[{}] 设置保证金模式失败: {}", exchange_name, e);
                }

                // 获取价格和交易规则
                let price = exchange.get_current_price(&symbol).await?;
                let rules = exchange.get_symbol_trading_rules(&symbol).await?;

                // 计算数量
                let quantity =
                    exchange.calculate_quantity_with_margin(margin, leverage, price, &rules);

                info!(
                    "[{}] 💰 计算: 保证金{}U × {}倍 = {}U, 价格:{}, 数量:{}",
                    exchange_name,
                    margin,
                    leverage,
                    margin * leverage as f64,
                    price,
                    quantity
                );

                // 执行开多
                let result = exchange
                    .open_long(&symbol, quantity, leverage, margin_type, dual_side)
                    .await?;

                info!(
                    "[{}] ✅ 开多成功: {} 订单ID: {}",
                    exchange_name, symbol, result.order_id
                );
                Ok(format!("[{}] 开多成功: {}", exchange_name, symbol))
            }

            SignalType::OpenShort(symbol) => {
                info!("[{}] 📉 执行开空: {}", exchange_name, symbol);

                // 设置杠杆和保证金模式
                if let Err(e) = exchange.set_leverage(&symbol, leverage).await {
                    warn!("[{}] 设置杠杆失败: {}", exchange_name, e);
                }

                if let Err(e) = exchange.set_margin_type(&symbol, margin_type).await {
                    warn!("[{}] 设置保证金模式失败: {}", exchange_name, e);
                }

                // 获取价格和交易规则
                let price = exchange.get_current_price(&symbol).await?;
                let rules = exchange.get_symbol_trading_rules(&symbol).await?;

                // 计算数量
                let quantity =
                    exchange.calculate_quantity_with_margin(margin, leverage, price, &rules);

                info!(
                    "[{}] 💰 计算: 保证金{}U × {}倍 = {}U, 价格:{}, 数量:{}",
                    exchange_name,
                    margin,
                    leverage,
                    margin * leverage as f64,
                    price,
                    quantity
                );

                // 执行开空
                let result = exchange
                    .open_short(&symbol, quantity, leverage, margin_type, dual_side)
                    .await?;

                info!(
                    "[{}] ✅ 开空成功: {} 订单ID: {}",
                    exchange_name, symbol, result.order_id
                );
                Ok(format!("[{}] 开空成功: {}", exchange_name, symbol))
            }

            SignalType::Close(symbol) => {
                info!("[{}] 🔄 执行平仓: {}", exchange_name, symbol);

                // 获取持仓
                let positions = exchange.get_positions().await?;

                if let Some(pos) = positions.iter().find(|p| p.symbol == symbol) {
                    let result = exchange
                        .close_position(&symbol, &pos.side, pos.size)
                        .await?;
                    info!(
                        "[{}] ✅ 平仓成功: {} {} {} 订单ID: {}",
                        exchange_name, symbol, pos.side, pos.size, result.order_id
                    );
                    Ok(format!("[{}] 平仓成功: {}", exchange_name, symbol))
                } else {
                    warn!("[{}] ⚠️  未找到持仓: {}", exchange_name, symbol);
                    Ok(format!("[{}] 无持仓", exchange_name))
                }
            }
        }
    }

    /// 获取所有交易所的账户信息
    pub async fn get_all_accounts(&self) -> Vec<(String, Result<AccountInfo>)> {
        let mut tasks = JoinSet::new();

        for exchange in &self.exchanges {
            let exchange_clone = Arc::clone(exchange);
            tasks.spawn(async move {
                let name = exchange_clone.get_exchange_name().to_string();
                let result = exchange_clone.get_account_info().await;
                (name, result)
            });
        }

        let mut results = Vec::new();
        while let Some(result) = tasks.join_next().await {
            if let Ok(r) = result {
                results.push(r);
            }
        }

        results
    }

    /// 获取所有交易所的持仓
    pub async fn get_all_positions(&self) -> Vec<(String, Result<Vec<Position>>)> {
        let mut tasks = JoinSet::new();

        for exchange in &self.exchanges {
            let exchange_clone = Arc::clone(exchange);
            tasks.spawn(async move {
                let name = exchange_clone.get_exchange_name().to_string();
                let result = exchange_clone.get_positions().await;
                (name, result)
            });
        }

        let mut results = Vec::new();
        while let Some(result) = tasks.join_next().await {
            if let Ok(r) = result {
                results.push(r);
            }
        }

        results
    }

    /// 打印所有账户摘要
    pub async fn print_accounts_summary(&self) {
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("📊 所有交易所账户摘要");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let accounts = self.get_all_accounts().await;
        let mut total_balance = 0.0;
        let mut total_pnl = 0.0;

        for (name, result) in accounts {
            match result {
                Ok(account) => {
                    info!("[{}]", name);
                    info!("  💰 总余额: {:.2} USDT", account.total_balance);
                    info!("  📈 可用余额: {:.2} USDT", account.available_balance);
                    info!("  📊 未实现盈亏: {:.2} USDT", account.unrealized_pnl);
                    info!("  🔒 已用保证金: {:.2} USDT", account.margin_used);

                    total_balance += account.total_balance;
                    total_pnl += account.unrealized_pnl;
                }
                Err(e) => {
                    error!("[{}] ❌ 获取账户信息失败: {}", name, e);
                }
            }
            info!("─────────────────────────────────────────");
        }

        info!("💎 总计余额: {:.2} USDT", total_balance);
        info!("💹 总计未实现盈亏: {:.2} USDT", total_pnl);
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    }

    /// 打印所有持仓
    pub async fn print_positions_summary(&self) {
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("📌 所有交易所持仓汇总");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let all_positions = self.get_all_positions().await;
        let mut total_positions = 0;
        let mut total_pnl = 0.0;

        for (name, result) in all_positions {
            match result {
                Ok(positions) => {
                    if positions.is_empty() {
                        info!("[{}] 无持仓", name);
                    } else {
                        info!("[{}] {} 个持仓:", name, positions.len());
                        for pos in &positions {
                            info!(
                                "  {} {} | 数量:{:.4} | 入场:{:.2} | 标记:{:.2} | PnL:{:.2} | 杠杆:{}x",
                                pos.symbol,
                                pos.side,
                                pos.size,
                                pos.entry_price,
                                pos.mark_price,
                                pos.pnl,
                                pos.leverage
                            );
                            total_pnl += pos.pnl;
                        }
                        total_positions += positions.len();
                    }
                }
                Err(e) => {
                    error!("[{}] ❌ 获取持仓失败: {}", name, e);
                }
            }
            info!("─────────────────────────────────────────");
        }

        info!("📊 总持仓数: {}", total_positions);
        info!("💹 总未实现盈亏: {:.2} USDT", total_pnl);
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    }
}
