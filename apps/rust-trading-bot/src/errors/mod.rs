//! Unified Error Handling
//!
//! 统一的错误类型定义，使用 thiserror 提供清晰的错误语义

use thiserror::Error;

/// 交易系统统一错误类型
#[derive(Error, Debug)]
pub enum TradingError {
    /// 交易所相关错误
    #[error("Exchange error: {0}")]
    Exchange(String),

    /// 订单执行错误
    #[error("Order execution failed: {0}")]
    OrderExecution(String),

    /// AI分析错误
    #[error("AI analysis failed: {0}")]
    AIAnalysis(String),

    /// 风控限制错误
    #[error("Risk limit exceeded: {0}")]
    RiskLimit(String),

    /// 数据库错误
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// 配置错误
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// 网络请求错误
    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),

    /// JSON序列化/反序列化错误
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// 通用错误
    #[error("General error: {0}")]
    General(#[from] anyhow::Error),

    /// 超时错误
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// 未找到
    #[error("Not found: {0}")]
    NotFound(String),

    /// 无效参数
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

/// 便捷的 Result 类型别名
pub type Result<T> = std::result::Result<T, TradingError>;

/// 错误码
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// 交易所错误
    Exchange = 1000,
    /// 订单执行错误
    OrderExecution = 2000,
    /// AI分析错误
    AIAnalysis = 3000,
    /// 风控错误
    RiskLimit = 4000,
    /// 数据库错误
    Database = 5000,
    /// 配置错误
    Config = 6000,
}

impl TradingError {
    /// 获取错误码
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::Exchange(_) => ErrorCode::Exchange,
            Self::OrderExecution(_) => ErrorCode::OrderExecution,
            Self::AIAnalysis(_) => ErrorCode::AIAnalysis,
            Self::RiskLimit(_) => ErrorCode::RiskLimit,
            Self::Database(_) => ErrorCode::Database,
            Self::InvalidConfig(_) => ErrorCode::Config,
            _ => ErrorCode::Exchange, // 默认
        }
    }

    /// 是否为可重试错误
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Network(_) | Self::Timeout(_) | Self::Exchange(_)
        )
    }

    /// 是否为致命错误
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            Self::InvalidConfig(_) | Self::Database(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = TradingError::RiskLimit("Max position exceeded".to_string());
        assert_eq!(err.to_string(), "Risk limit exceeded: Max position exceeded");
    }

    #[test]
    fn test_error_code() {
        let err = TradingError::AIAnalysis("Failed".to_string());
        assert_eq!(err.code(), ErrorCode::AIAnalysis);
    }

    #[test]
    fn test_retryable() {
        let err = TradingError::Timeout("5s".to_string());
        assert!(err.is_retryable());

        let err = TradingError::InvalidConfig("Bad config".to_string());
        assert!(!err.is_retryable());
    }
}
