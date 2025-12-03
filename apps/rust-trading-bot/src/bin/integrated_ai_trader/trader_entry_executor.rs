use super::*;
use crate::modules::types::EntryExecutionRequest;

impl IntegratedAITrader {
    /// 统一的试探建仓执行逻辑，便于被不同入口共享
    pub(super) async fn execute_ai_trial_entry(
        &self,
        req: EntryExecutionRequest<'_>,
    ) -> Result<()> {
        self.entry_manager.execute_ai_trial_entry(req).await
    }
}
