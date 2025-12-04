//! Analysis Service
//!
//! AI分析服务 - 负责调用AI进行入场和持仓分析

use anyhow::Result;
use log::{info, warn};
use std::sync::Arc;

use crate::ai_core::deepseek::DeepSeekClient;
use crate::ai_core::gemini::GeminiClient;
use crate::analysis::technical::TechnicalAnalyzer;
use crate::trading_core::signals::FundAlert;

/// AI分析服务
pub struct AnalysisService {
    deepseek: Arc<DeepSeekClient>,
    gemini: Arc<GeminiClient>,
    technical_analyzer: Arc<TechnicalAnalyzer>,
}

impl AnalysisService {
    /// 创建新的分析服务
    pub fn new(
        deepseek: Arc<DeepSeekClient>,
        gemini: Arc<GeminiClient>,
        technical_analyzer: Arc<TechnicalAnalyzer>,
    ) -> Self {
        Self {
            deepseek,
            gemini,
            technical_analyzer,
        }
    }

    /// 分析入场机会
    pub async fn analyze_entry(&self, alert: &FundAlert) -> Result<EntryDecision> {
        info!("🤖 开始AI入场分析: {}", alert.coin);

        // 这里将来会实现完整的AI分析逻辑
        // 当前返回占位结果
        Ok(EntryDecision {
            symbol: alert.coin.clone(),
            should_enter: false,
            confidence: "LOW".to_string(),
            reason: "Analysis service placeholder".to_string(),
        })
    }

    /// 分析持仓决策
    pub async fn analyze_position(&self, symbol: &str) -> Result<PositionDecision> {
        info!("🤖 开始持仓分析: {}", symbol);

        // 占位实现
        Ok(PositionDecision {
            symbol: symbol.to_string(),
            action: "HOLD".to_string(),
            confidence: "LOW".to_string(),
            reason: "Analysis service placeholder".to_string(),
        })
    }

    /// 批量分析多个持仓
    pub async fn batch_analyze_positions(&self, symbols: &[String]) -> Result<Vec<PositionDecision>> {
        info!("🤖 批量分析 {} 个持仓", symbols.len());

        let mut decisions = Vec::new();
        for symbol in symbols {
            match self.analyze_position(symbol).await {
                Ok(decision) => decisions.push(decision),
                Err(e) => warn!("⚠️ 分析 {} 失败: {}", symbol, e),
            }
        }

        Ok(decisions)
    }
}

/// 入场决策
#[derive(Debug, Clone)]
pub struct EntryDecision {
    pub symbol: String,
    pub should_enter: bool,
    pub confidence: String,
    pub reason: String,
}

/// 持仓决策
#[derive(Debug, Clone)]
pub struct PositionDecision {
    pub symbol: String,
    pub action: String, // "HOLD", "CLOSE", "REDUCE"
    pub confidence: String,
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analysis_service_creation() {
        let deepseek = Arc::new(DeepSeekClient::new("test_key".to_string()));
        let gemini = Arc::new(GeminiClient::new("test_key".to_string()));
        let analyzer = Arc::new(TechnicalAnalyzer::new());

        let _service = AnalysisService::new(deepseek, gemini, analyzer);
        assert!(true);
    }
}
