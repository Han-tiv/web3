use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use log::{info, warn};
use tokio::task::JoinSet;

use super::ai_trait::{AIProvider, EntryContext, EntryDecision, PositionContext, PositionDecision};

/// 共识配置。
#[derive(Debug, Clone)]
pub struct DecisionEngineConfig {
    /// 达成共识所需的最少一致票数。
    pub quorum: usize,
    /// 至少需要多少个 AI 提供者返回结果。
    pub min_providers: usize,
    /// 是否在未达成共识时立即报错（严格模式）。
    pub strict: bool,
}

impl Default for DecisionEngineConfig {
    fn default() -> Self {
        Self {
            quorum: 2,
            min_providers: 1,
            strict: false,
        }
    }
}

/// AI 决策引擎，负责并发调用多个提供者并产生共识结果。
pub struct DecisionEngine {
    providers: Vec<Arc<dyn AIProvider>>,
    config: DecisionEngineConfig,
}

impl DecisionEngine {
    pub fn new(providers: Vec<Arc<dyn AIProvider>>, config: DecisionEngineConfig) -> Self {
        Self {
            providers,
            config: DecisionEngineConfig {
                quorum: config.quorum.max(1),
                min_providers: config.min_providers.max(1),
                strict: config.strict,
            },
        }
    }

    pub fn with_default_config(providers: Vec<Arc<dyn AIProvider>>) -> Self {
        Self::new(providers, DecisionEngineConfig::default())
    }

    pub fn add_provider(&mut self, provider: Arc<dyn AIProvider>) {
        self.providers.push(provider);
    }

    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }

    pub fn provider_names(&self) -> Vec<&'static str> {
        self.providers.iter().map(|p| p.name()).collect()
    }

    pub async fn entry_consensus(&self, ctx: EntryContext) -> Result<EntryConsensus> {
        let responses = self.collect_entry_decisions(ctx).await?;
        self.determine_entry_consensus(responses)
    }

    pub async fn position_consensus(&self, ctx: PositionContext) -> Result<PositionConsensus> {
        let responses = self.collect_position_decisions(ctx).await?;
        self.determine_position_consensus(responses)
    }

    async fn collect_entry_decisions(&self, ctx: EntryContext) -> Result<Vec<EntryDecision>> {
        if self.providers.is_empty() {
            return Err(anyhow!("未注册 AI Provider，无法执行入场分析"));
        }

        let mut join_set = JoinSet::new();
        for provider in &self.providers {
            let provider_clone = provider.clone();
            let provider_name = provider.name().to_string();
            let cloned_ctx = ctx.clone();

            join_set.spawn(async move {
                let result = provider_clone.analyze_entry(&cloned_ctx).await;
                (provider_name, result)
            });
        }

        self.collect_join_results(join_set, self.config.min_providers)
            .await
    }

    async fn collect_position_decisions(
        &self,
        ctx: PositionContext,
    ) -> Result<Vec<PositionDecision>> {
        if self.providers.is_empty() {
            return Err(anyhow!("未注册 AI Provider，无法执行持仓分析"));
        }

        let mut join_set = JoinSet::new();
        for provider in &self.providers {
            let provider_clone = provider.clone();
            let provider_name = provider.name().to_string();
            let cloned_ctx = ctx.clone();

            join_set.spawn(async move {
                let result = provider_clone.analyze_position(&cloned_ctx).await;
                (provider_name, result)
            });
        }

        self.collect_join_results(join_set, self.config.min_providers)
            .await
    }

    async fn collect_join_results<T: Send + 'static>(
        &self,
        mut join_set: JoinSet<(String, Result<T>)>,
        min_providers: usize,
    ) -> Result<Vec<T>> {
        let mut results = Vec::new();

        while let Some(join_result) = join_set.join_next().await {
            match join_result {
                Ok((_provider_name, Ok(decision))) => results.push(decision),
                Ok((provider_name, Err(err))) => {
                    warn!("⚠️ AI提供者 {} 返回错误: {}", provider_name, err);
                }
                Err(err) => {
                    warn!("⚠️ AI任务执行失败: {}", err);
                }
            }
        }

        if results.len() < min_providers {
            return Err(anyhow!(
                "AI响应数量不足: {} / {}",
                results.len(),
                min_providers
            ));
        }

        Ok(results)
    }

    fn determine_entry_consensus(&self, decisions: Vec<EntryDecision>) -> Result<EntryConsensus> {
        if decisions.is_empty() {
            return Err(anyhow!("无法形成入场共识: 所有 AI 提供者均失败或拒绝输出"));
        }

        let (vote_map, final_decision) =
            self.pick_consensus(decisions.clone(), self.config.quorum, |d| {
                (d.action.key(), d.confidence_score())
            })?;

        info!(
            "🤝 入场共识: {} ({} 票)",
            final_decision.action.as_str(),
            vote_map
                .get(&final_decision.action.key())
                .copied()
                .unwrap_or(1)
        );

        Ok(EntryConsensus {
            final_decision,
            decisions,
            votes: vote_map,
        })
    }

    fn determine_position_consensus(
        &self,
        decisions: Vec<PositionDecision>,
    ) -> Result<PositionConsensus> {
        if decisions.is_empty() {
            return Err(anyhow!("无法形成持仓共识: 所有 AI 提供者均失败或拒绝输出"));
        }

        let (vote_map, final_decision) =
            self.pick_consensus(decisions.clone(), self.config.quorum, |d| {
                (d.action.key(), d.confidence_score())
            })?;

        info!(
            "🤝 持仓共识: {} ({} 票)",
            final_decision.action.as_str(),
            vote_map
                .get(&final_decision.action.key())
                .copied()
                .unwrap_or(1)
        );

        Ok(PositionConsensus {
            final_decision,
            decisions,
            votes: vote_map,
        })
    }

    fn pick_consensus<T, F>(
        &self,
        decisions: Vec<T>,
        quorum: usize,
        key_fn: F,
    ) -> Result<(HashMap<String, usize>, T)>
    where
        T: Clone,
        F: Fn(&T) -> (String, u8),
    {
        let mut grouped: HashMap<String, Vec<T>> = HashMap::new();
        let mut vote_map: HashMap<String, usize> = HashMap::new();

        for decision in &decisions {
            let (key, _) = key_fn(decision);
            vote_map
                .entry(key.clone())
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }

        for decision in decisions {
            let (key, _) = key_fn(&decision);
            grouped.entry(key).or_default().push(decision);
        }

        let mut best_key = None;
        let mut best_len = 0usize;
        let mut best_confidence = 0usize;

        for (key, group) in &grouped {
            let len = group.len();
            let confidence: usize = group.iter().map(|item| key_fn(item).1 as usize).sum();

            if len > best_len || (len == best_len && confidence > best_confidence) {
                best_key = Some(key.clone());
                best_len = len;
                best_confidence = confidence;
            }
        }

        let best_key = match best_key {
            Some(key) => key,
            None => {
                return Err(anyhow!("计算共识失败：没有有效决策"));
            }
        };

        let mut best_group = grouped
            .remove(&best_key)
            .ok_or_else(|| anyhow!("共识分组丢失"))?;

        if best_group.len() < quorum && self.config.strict {
            return Err(anyhow!("未达到共识阈值: {} / {}", best_group.len(), quorum));
        }

        best_group.sort_by_key(|decision| key_fn(decision).1);
        let final_decision = best_group.pop().ok_or_else(|| anyhow!("共识分组为空"))?;

        Ok((vote_map, final_decision))
    }
}

/// 入场共识结果。
#[derive(Debug, Clone)]
pub struct EntryConsensus {
    pub final_decision: EntryDecision,
    pub decisions: Vec<EntryDecision>,
    pub votes: HashMap<String, usize>,
}

/// 持仓共识结果。
#[derive(Debug, Clone)]
pub struct PositionConsensus {
    pub final_decision: PositionDecision,
    pub decisions: Vec<PositionDecision>,
    pub votes: HashMap<String, usize>,
}
