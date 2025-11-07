// 交易所统一接口定义
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use log::{info, warn};
use serde::{Deserialize, Serialize};

/// 统一的持仓信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub side: String,     // "LONG" or "SHORT"
    pub size: f64,        // 持仓数量（绝对值）
    pub entry_price: f64, // 开仓价格
    pub mark_price: f64,  // 标记价格
    pub pnl: f64,         // 未实现盈亏
    pub leverage: i32,    // 杠杆倍数
    pub margin: f64,      // 保证金
}

/// 统一的账户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub total_balance: f64,     // 总余额
    pub available_balance: f64, // 可用余额
    pub unrealized_pnl: f64,    // 未实现盈亏
    pub margin_used: f64,       // 已用保证金
}

/// 交易规则
#[derive(Debug, Clone)]
pub struct TradingRules {
    pub step_size: f64,          // 数量步长
    pub min_qty: f64,            // 最小数量
    pub quantity_precision: i32, // 数量精度
    pub price_precision: i32,    // 价格精度
    pub tick_size: f64,          // 价格步长 (PRICE_FILTER)
}

/// 订单结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResult {
    pub order_id: String,
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
    pub price: f64,
    pub status: String,
}

/// 交易所客户端统一接口
#[async_trait]
pub trait ExchangeClient: Send + Sync {
    /// 获取交易所名称
    fn get_exchange_name(&self) -> &str;

    /// 获取当前持仓列表
    async fn get_positions(&self) -> Result<Vec<Position>>;

    /// 获取单个币种的持仓
    async fn get_position(&self, symbol: &str) -> Result<Option<Position>> {
        let positions = self.get_positions().await?;
        let total_positions = positions.len();
        let position = positions.into_iter().find(|p| p.symbol == symbol);

        if let Some(pos) = position.as_ref() {
            info!(
                "🔍 已定位{}持仓: 方向={} 数量={:.6} (总持仓数={})",
                symbol, pos.side, pos.size, total_positions
            );
        } else {
            warn!("⚠️  未找到{}持仓, 当前持仓总数={}", symbol, total_positions);
        }

        Ok(position)
    }

    /// 获取账户信息
    async fn get_account_info(&self) -> Result<AccountInfo>;

    /// 获取当前市场价格
    async fn get_current_price(&self, symbol: &str) -> Result<f64>;

    /// 获取交易对交易规则
    async fn get_symbol_trading_rules(&self, symbol: &str) -> Result<TradingRules>;

    /// 设置杠杆
    async fn set_leverage(&self, symbol: &str, leverage: u32) -> Result<()>;

    /// 设置保证金模式
    async fn set_margin_type(&self, symbol: &str, margin_type: &str) -> Result<()>;

    /// 设置持仓模式
    async fn set_position_mode(&self, dual_side: bool) -> Result<()>;

    /// 开多仓
    async fn open_long(
        &self,
        symbol: &str,
        quantity: f64,
        leverage: u32,
        margin_type: &str,
        dual_side: bool,
    ) -> Result<OrderResult>;

    /// 开空仓
    async fn open_short(
        &self,
        symbol: &str,
        quantity: f64,
        leverage: u32,
        margin_type: &str,
        dual_side: bool,
    ) -> Result<OrderResult>;

    /// 平仓
    async fn close_position(&self, symbol: &str, side: &str, size: f64) -> Result<OrderResult>;

    /// 获取K线数据
    ///
    /// # 参数
    /// - `symbol`: 交易对符号 (如 "BTCUSDT")
    /// - `interval`: K线周期 ("1m", "5m", "15m", "1h", "4h", "1d")
    /// - `limit`: 返回K线数量 (默认100根)
    ///
    /// # 返回
    /// Vec<Vec<f64>>: [[timestamp, open, high, low, close, volume], ...]
    async fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Vec<f64>>> {
        Err(anyhow!(
            "get_klines 未实现: {} {} {:?}",
            self.get_exchange_name(),
            symbol,
            limit
        ))
    }

    /// 调整持仓数量 (加仓/减仓)
    ///
    /// # 参数
    /// - `symbol`: 交易对符号
    /// - `side`: 持仓方向 ("LONG" or "SHORT")
    /// - `quantity_delta`: 调整数量 (正数=加仓, 负数=减仓)
    /// - `leverage`: 杠杆倍数
    /// - `margin_type`: 保证金模式
    ///
    /// # 示例
    /// ```
    /// // 加仓50%
    /// adjust_position("BTCUSDT", "LONG", 0.5, 5, "CROSSED").await?;
    ///
    /// // 减仓30%
    /// adjust_position("BTCUSDT", "LONG", -0.3, 5, "CROSSED").await?;
    /// ```
    async fn adjust_position(
        &self,
        symbol: &str,
        side: &str,
        quantity_delta: f64,
        leverage: u32,
        margin_type: &str,
    ) -> Result<OrderResult> {
        Err(anyhow!(
            "adjust_position 未实现: {} {} {}",
            self.get_exchange_name(),
            symbol,
            side
        ))
    }

    /// 根据保证金和杠杆计算数量
    fn calculate_quantity_with_margin(
        &self,
        margin_usdt: f64,
        leverage: u32,
        price: f64,
        rules: &TradingRules,
    ) -> f64 {
        let notional = margin_usdt * leverage as f64;
        let raw_quantity = notional / price;

        // 按步长对齐
        let quantity = (raw_quantity / rules.step_size).floor() * rules.step_size;

        // 确保不小于最小数量
        quantity.max(rules.min_qty)
    }
}

/// 交易所类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExchangeType {
    Binance,
    Okx,
    Bitget,
    Bybit,
    Gate,
}

impl ExchangeType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "binance" => Some(ExchangeType::Binance),
            "okx" => Some(ExchangeType::Okx),
            "bitget" => Some(ExchangeType::Bitget),
            "bybit" => Some(ExchangeType::Bybit),
            "gate" => Some(ExchangeType::Gate),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            ExchangeType::Binance => "binance",
            ExchangeType::Okx => "okx",
            ExchangeType::Bitget => "bitget",
            ExchangeType::Bybit => "bybit",
            ExchangeType::Gate => "gate",
        }
    }
}
