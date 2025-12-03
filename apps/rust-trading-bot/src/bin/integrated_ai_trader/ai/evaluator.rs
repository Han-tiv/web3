use anyhow::Result;
use log::{error, warn};
use rust_trading_bot::{deepseek_client::PositionManagementDecision, gemini_client::GeminiClient};
use std::sync::Arc;

use super::super::{PositionAction, PositionContextRequest, PositionEvaluationStep};
use crate::trader::build_position_prompt_v2;

use super::{context_builder::ContextBuilder, decision_handler::DecisionHandler};

/// 串联上下文构建、Gemini 调用与动作生成
pub struct PositionEvaluator {
    gemini_client: Arc<GeminiClient>,
    context_builder: ContextBuilder,
    decision_handler: DecisionHandler,
}

impl PositionEvaluator {
    pub fn new(
        gemini_client: Arc<GeminiClient>,
        context_builder: ContextBuilder,
        decision_handler: DecisionHandler,
    ) -> Self {
        Self {
            gemini_client,
            context_builder,
            decision_handler,
        }
    }

    pub fn context_builder(&self) -> &ContextBuilder {
        &self.context_builder
    }

    pub fn decision_handler(&self) -> &DecisionHandler {
        &self.decision_handler
    }

    pub async fn evaluate(
        &self,
        req: PositionContextRequest<'_>,
    ) -> Result<Option<PositionAction>> {
        match self.context_builder.prepare_position_context(req).await?
        {
            PositionEvaluationStep::Skip => Ok(None),
            PositionEvaluationStep::Immediate(action) => Ok(Some(action)),
            PositionEvaluationStep::Context(ctx) => {
                let prompt = build_position_prompt_v2(&ctx);

                let ai_decision_result = tokio::time::timeout(
                    tokio::time::Duration::from_secs(180),
                    self.gemini_client.analyze_position_management(&prompt),
                )
                .await;

                let ai_decision: PositionManagementDecision = match ai_decision_result {
                    Ok(Ok(decision)) => decision,
                    Ok(Err(e)) => {
                        error!("❌ AI持仓评估失败: {}, 保持持仓", e);
                        return Ok(None);
                    }
                    Err(_) => {
                        warn!("⚠️  AI持仓评估超时, 保持持仓");
                        return Ok(None);
                    }
                };

                self.decision_handler
                    .handle_decision(&ctx, &ai_decision)
                    .await
            }
        }
    }
}
