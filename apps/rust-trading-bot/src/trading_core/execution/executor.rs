use anyhow::{Context, Result};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::exchange_trait::{ExchangeClient, Position};
use crate::position_coordinator::{TradeAction, TradeActionType};

/// 速率限制器
struct RateLimiter {
    last_call: tokio::time::Instant,
    min_interval: Duration,
}

impl RateLimiter {
    fn new(min_interval_ms: u64) -> Self {
        Self {
            last_call: tokio::time::Instant::now(),
            min_interval: Duration::from_millis(min_interval_ms),
        }
    }

    async fn wait(&mut self) {
        let elapsed = self.last_call.elapsed();
        if elapsed < self.min_interval {
            let wait_time = self.min_interval - elapsed;
            tokio::time::sleep(wait_time).await;
        }
        self.last_call = tokio::time::Instant::now();
    }
}

/// 交易执行器配置
#[derive(Debug, Clone)]
pub struct TradeExecutorConfig {
    /// API调用最小间隔（毫秒）
    pub min_api_interval_ms: u64,
    /// 单笔最大资金（USDT）
    pub max_position_usdt: f64,
    /// 基础开仓资金（USDT）
    pub base_position_usdt: f64,
    /// 保证金模式（"cross" or "isolated"）
    pub margin_type: String,
    /// 是否启用双向持仓
    pub dual_side_position: bool,
}

impl Default for TradeExecutorConfig {
    fn default() -> Self {
        Self {
            min_api_interval_ms: 500,
            max_position_usdt: 100.0,
            base_position_usdt: 10.0,
            margin_type: "cross".to_string(),
            dual_side_position: false,
        }
    }
}

/// 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub symbol: String,
    pub action_type: TradeActionType,
    pub success: bool,
    pub error_message: Option<String>,
    pub order_id: Option<String>,
    pub executed_quantity: f64,
    pub executed_price: f64,
}

/// 执行统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStats {
    pub total_actions: usize,
    pub successful: usize,
    pub failed: usize,
    pub skipped: usize,
}

/// 交易执行器
pub struct TradeExecutor {
    config: TradeExecutorConfig,
    exchange: Arc<dyn ExchangeClient>,
    rate_limiter: Arc<Mutex<RateLimiter>>,
}

impl TradeExecutor {
    pub fn new(config: TradeExecutorConfig, exchange: Arc<dyn ExchangeClient>) -> Self {
        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(config.min_api_interval_ms)));

        Self {
            config,
            exchange,
            rate_limiter,
        }
    }

    /// 执行交易计划
    pub async fn execute_plan(
        &self,
        actions: Vec<TradeAction>,
    ) -> (Vec<ExecutionResult>, ExecutionStats) {
        info!("═══════════════════════════════════════════");
        info!("开始执行交易计划: {} 个动作", actions.len());
        info!("═══════════════════════════════════════════");

        let mut results = Vec::new();
        let mut stats = ExecutionStats {
            total_actions: actions.len(),
            successful: 0,
            failed: 0,
            skipped: 0,
        };

        for (idx, action) in actions.iter().enumerate() {
            info!(
                "\n[{}/{}] 执行: {:?} - {}",
                idx + 1,
                actions.len(),
                action.action_type,
                action.symbol
            );
            info!(
                "  优先级: {:?} | 信心: {} | 原因: {}",
                action.priority, action.ai_confidence, action.reason
            );

            let result = self.execute_single(action).await;

            if result.success {
                stats.successful += 1;
                info!("✅ 执行成功");
            } else {
                stats.failed += 1;
                error!(
                    "❌ 执行失败: {}",
                    result
                        .error_message
                        .as_ref()
                        .unwrap_or(&"未知错误".to_string())
                );
            }

            results.push(result);

            // 等待速率限制
            self.rate_limiter.lock().await.wait().await;
        }

        info!("\n═══════════════════════════════════════════");
        info!("交易计划执行完成");
        info!(
            "  总动作: {} | 成功: {} | 失败: {} | 跳过: {}",
            stats.total_actions, stats.successful, stats.failed, stats.skipped
        );
        info!("═══════════════════════════════════════════\n");

        (results, stats)
    }

    /// 执行单个交易动作
    async fn execute_single(&self, action: &TradeAction) -> ExecutionResult {
        let result = match action.action_type {
            TradeActionType::OpenLong => self.handle_open_long(action).await,
            TradeActionType::OpenShort => self.handle_open_short(action).await,
            TradeActionType::CloseLong => self.handle_close_long(action).await,
            TradeActionType::CloseShort => self.handle_close_short(action).await,
            TradeActionType::AddLong => self.handle_add_long(action).await,
            TradeActionType::AddShort => self.handle_add_short(action).await,
            TradeActionType::ReduceLong => self.handle_reduce_long(action).await,
            TradeActionType::ReduceShort => self.handle_reduce_short(action).await,
            TradeActionType::ReverseToLong => self.handle_reverse_to_long(action).await,
            TradeActionType::ReverseToShort => self.handle_reverse_to_short(action).await,
            TradeActionType::Hold => Ok(ExecutionResult {
                symbol: action.symbol.clone(),
                action_type: action.action_type.clone(),
                success: true,
                error_message: None,
                order_id: None,
                executed_quantity: 0.0,
                executed_price: 0.0,
            }),
        };

        result.unwrap_or_else(|e| ExecutionResult {
            symbol: action.symbol.clone(),
            action_type: action.action_type.clone(),
            success: false,
            error_message: Some(e.to_string()),
            order_id: None,
            executed_quantity: 0.0,
            executed_price: 0.0,
        })
    }

    /// 开多仓
    async fn handle_open_long(&self, action: &TradeAction) -> Result<ExecutionResult> {
        let price = self.exchange.get_current_price(&action.symbol).await?;
        let quantity = self.calculate_quantity(&action.symbol, price).await?;

        let order = self
            .exchange
            .open_long(
                &action.symbol,
                quantity,
                action.leverage,
                &self.config.margin_type,
                self.config.dual_side_position,
            )
            .await?;

        Ok(ExecutionResult {
            symbol: action.symbol.clone(),
            action_type: action.action_type.clone(),
            success: true,
            error_message: None,
            order_id: Some(order.order_id),
            executed_quantity: order.quantity,
            executed_price: order.price,
        })
    }

    /// 开空仓
    async fn handle_open_short(&self, action: &TradeAction) -> Result<ExecutionResult> {
        let price = self.exchange.get_current_price(&action.symbol).await?;
        let quantity = self.calculate_quantity(&action.symbol, price).await?;

        let order = self
            .exchange
            .open_short(
                &action.symbol,
                quantity,
                action.leverage,
                &self.config.margin_type,
                self.config.dual_side_position,
            )
            .await?;

        Ok(ExecutionResult {
            symbol: action.symbol.clone(),
            action_type: action.action_type.clone(),
            success: true,
            error_message: None,
            order_id: Some(order.order_id),
            executed_quantity: order.quantity,
            executed_price: order.price,
        })
    }

    /// 平多仓
    async fn handle_close_long(&self, action: &TradeAction) -> Result<ExecutionResult> {
        let position = self.get_position(&action.symbol, "LONG").await?;

        let order = self
            .exchange
            .close_position(&action.symbol, "LONG", position.size)
            .await?;

        Ok(ExecutionResult {
            symbol: action.symbol.clone(),
            action_type: action.action_type.clone(),
            success: true,
            error_message: None,
            order_id: Some(order.order_id),
            executed_quantity: order.quantity,
            executed_price: order.price,
        })
    }

    /// 平空仓
    async fn handle_close_short(&self, action: &TradeAction) -> Result<ExecutionResult> {
        let position = self.get_position(&action.symbol, "SHORT").await?;

        let order = self
            .exchange
            .close_position(&action.symbol, "SHORT", position.size)
            .await?;

        Ok(ExecutionResult {
            symbol: action.symbol.clone(),
            action_type: action.action_type.clone(),
            success: true,
            error_message: None,
            order_id: Some(order.order_id),
            executed_quantity: order.quantity,
            executed_price: order.price,
        })
    }

    /// 加多仓
    async fn handle_add_long(&self, action: &TradeAction) -> Result<ExecutionResult> {
        let order = self
            .exchange
            .open_long(
                &action.symbol,
                action.quantity,
                action.leverage,
                &self.config.margin_type,
                self.config.dual_side_position,
            )
            .await?;

        Ok(ExecutionResult {
            symbol: action.symbol.clone(),
            action_type: action.action_type.clone(),
            success: true,
            error_message: None,
            order_id: Some(order.order_id),
            executed_quantity: order.quantity,
            executed_price: order.price,
        })
    }

    /// 加空仓
    async fn handle_add_short(&self, action: &TradeAction) -> Result<ExecutionResult> {
        let order = self
            .exchange
            .open_short(
                &action.symbol,
                action.quantity,
                action.leverage,
                &self.config.margin_type,
                self.config.dual_side_position,
            )
            .await?;

        Ok(ExecutionResult {
            symbol: action.symbol.clone(),
            action_type: action.action_type.clone(),
            success: true,
            error_message: None,
            order_id: Some(order.order_id),
            executed_quantity: order.quantity,
            executed_price: order.price,
        })
    }

    /// 减多仓
    async fn handle_reduce_long(&self, action: &TradeAction) -> Result<ExecutionResult> {
        let order = self
            .exchange
            .close_position(&action.symbol, "LONG", action.quantity)
            .await?;

        Ok(ExecutionResult {
            symbol: action.symbol.clone(),
            action_type: action.action_type.clone(),
            success: true,
            error_message: None,
            order_id: Some(order.order_id),
            executed_quantity: order.quantity,
            executed_price: order.price,
        })
    }

    /// 减空仓
    async fn handle_reduce_short(&self, action: &TradeAction) -> Result<ExecutionResult> {
        let order = self
            .exchange
            .close_position(&action.symbol, "SHORT", action.quantity)
            .await?;

        Ok(ExecutionResult {
            symbol: action.symbol.clone(),
            action_type: action.action_type.clone(),
            success: true,
            error_message: None,
            order_id: Some(order.order_id),
            executed_quantity: order.quantity,
            executed_price: order.price,
        })
    }

    /// 反向到多（先平空再开多）
    async fn handle_reverse_to_long(&self, action: &TradeAction) -> Result<ExecutionResult> {
        // 1. 平空仓
        let position = self.get_position(&action.symbol, "SHORT").await?;
        self.exchange
            .close_position(&action.symbol, "SHORT", position.size)
            .await?;

        info!("  ✓ 已平空仓 {} {}", action.symbol, position.size);

        // 等待1秒
        tokio::time::sleep(Duration::from_secs(1)).await;

        // 2. 开多仓
        let price = self.exchange.get_current_price(&action.symbol).await?;
        let quantity = self.calculate_quantity(&action.symbol, price).await?;

        let order = self
            .exchange
            .open_long(
                &action.symbol,
                quantity,
                action.leverage,
                &self.config.margin_type,
                self.config.dual_side_position,
            )
            .await?;

        info!("  ✓ 已开多仓 {} {}", action.symbol, order.quantity);

        Ok(ExecutionResult {
            symbol: action.symbol.clone(),
            action_type: action.action_type.clone(),
            success: true,
            error_message: None,
            order_id: Some(order.order_id),
            executed_quantity: order.quantity,
            executed_price: order.price,
        })
    }

    /// 反向到空（先平多再开空）
    async fn handle_reverse_to_short(&self, action: &TradeAction) -> Result<ExecutionResult> {
        // 1. 平多仓
        let position = self.get_position(&action.symbol, "LONG").await?;
        self.exchange
            .close_position(&action.symbol, "LONG", position.size)
            .await?;

        info!("  ✓ 已平多仓 {} {}", action.symbol, position.size);

        // 等待1秒
        tokio::time::sleep(Duration::from_secs(1)).await;

        // 2. 开空仓
        let price = self.exchange.get_current_price(&action.symbol).await?;
        let quantity = self.calculate_quantity(&action.symbol, price).await?;

        let order = self
            .exchange
            .open_short(
                &action.symbol,
                quantity,
                action.leverage,
                &self.config.margin_type,
                self.config.dual_side_position,
            )
            .await?;

        info!("  ✓ 已开空仓 {} {}", action.symbol, order.quantity);

        Ok(ExecutionResult {
            symbol: action.symbol.clone(),
            action_type: action.action_type.clone(),
            success: true,
            error_message: None,
            order_id: Some(order.order_id),
            executed_quantity: order.quantity,
            executed_price: order.price,
        })
    }

    /// 计算开仓数量
    async fn calculate_quantity(&self, symbol: &str, price: f64) -> Result<f64> {
        let rules = self.exchange.get_symbol_trading_rules(symbol).await?;

        let notional = self.config.base_position_usdt;
        let raw_quantity = notional / price;

        // 按步长对齐
        let quantity = (raw_quantity / rules.step_size).floor() * rules.step_size;

        // 确保不小于最小数量
        let final_quantity = quantity.max(rules.min_qty);

        Ok(final_quantity)
    }

    /// 获取持仓
    async fn get_position(&self, symbol: &str, side: &str) -> Result<Position> {
        let positions = self.exchange.get_positions().await?;

        positions
            .into_iter()
            .find(|p| p.symbol == symbol && p.side.to_uppercase() == side.to_uppercase())
            .context(format!("未找到 {} {} 持仓", symbol, side))
    }
}
