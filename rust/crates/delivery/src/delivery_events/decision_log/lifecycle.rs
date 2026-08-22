use super::{
    retention, trim, ActionBinding, DecisionLog, DecisionResolution, DecisionToken,
    RequestDecisionBinding,
};
use ghostr_engine::adaptive::{DecisionOutcome, DecisionRecord, ResourceCost};
use ghostr_engine::ActionId;
use std::sync::Arc;

impl DecisionLog {
    pub(super) fn bind(
        &self,
        token: &DecisionToken,
        action: ActionId,
        observed_at_ms: u64,
    ) -> bool {
        if !token.belongs_to(&self.store) {
            return false;
        }
        let mut store = self.lock();
        if store.actions.contains_key(&action) {
            return false;
        }
        if !bind_record(&mut store.records, token.sequence, action) {
            return false;
        }
        store.actions.insert(
            action,
            ActionBinding {
                sequence: token.sequence,
                started_at_ms: observed_at_ms,
            },
        );
        true
    }

    pub(super) fn resolve(
        &self,
        action: ActionId,
        outcome: DecisionOutcome,
        observed_at_ms: u64,
    ) -> Option<DecisionResolution> {
        self.resolve_observation(action, outcome, None, observed_at_ms)
    }

    pub(super) fn resolve_with_resources(
        &self,
        action: ActionId,
        outcome: DecisionOutcome,
        resources: ResourceCost,
        observed_at_ms: u64,
    ) -> Option<DecisionResolution> {
        self.resolve_observation(action, outcome, Some(resources), observed_at_ms)
    }

    fn resolve_observation(
        &self,
        action: ActionId,
        outcome: DecisionOutcome,
        resources: Option<ResourceCost>,
        observed_at_ms: u64,
    ) -> Option<DecisionResolution> {
        if outcome == DecisionOutcome::Pending {
            return None;
        }
        let mut store = self.lock();
        let binding = store.actions.get(&action).copied()?;
        let elapsed_ms = observed_at_ms.saturating_sub(binding.started_at_ms);
        let outcome = with_elapsed(outcome, elapsed_ms);
        let (decision_action, warp_action) =
            retention::resolve(&mut store.records, binding.sequence, outcome, resources)?;
        store.actions.remove(&action);
        store.completed.push_back(binding.sequence);
        trim(&mut store);
        Some(DecisionResolution {
            action: decision_action,
            warp_action,
            elapsed_ms,
        })
    }

    pub(super) fn bind_request(
        &self,
        token: &DecisionToken,
        binding: RequestDecisionBinding<'_>,
    ) -> bool {
        if !token.belongs_to(&self.store) {
            return false;
        }
        bind_request_record(&mut self.lock(), token.sequence, binding, &self.privacy)
    }

    pub(super) fn resolve_token(&self, token: &DecisionToken, outcome: DecisionOutcome) -> bool {
        if outcome == DecisionOutcome::Pending || !token.belongs_to(&self.store) {
            return false;
        }
        let mut store = self.lock();
        let claimed = store.claimed.contains(&token.sequence);
        let Some(sequence) =
            retention::resolve_unbound(&mut store.records, claimed, token.sequence, outcome)
        else {
            return false;
        };
        store.completed.push_back(sequence);
        trim(&mut store);
        true
    }
}

fn bind_request_record(
    store: &mut super::DecisionStore,
    sequence: u64,
    binding: RequestDecisionBinding<'_>,
    privacy: &ghostr_engine::adaptive::DecisionPrivacy,
) -> bool {
    if store.actions.contains_key(&binding.action) {
        return false;
    }
    let Some(record) = record_mut(&mut store.records, sequence) else {
        return false;
    };
    if !record.bind_executed_action(binding.action, binding.request, privacy) {
        return false;
    }
    store.actions.insert(
        binding.action,
        ActionBinding {
            sequence,
            started_at_ms: binding.observed_at_ms,
        },
    );
    true
}

impl DecisionToken {
    pub(super) fn new(sequence: u64, owner: &Arc<std::sync::Mutex<super::DecisionStore>>) -> Self {
        Self {
            sequence,
            owner: Arc::downgrade(owner),
            armed: true,
        }
    }

    pub(super) fn belongs_to(&self, store: &Arc<std::sync::Mutex<super::DecisionStore>>) -> bool {
        std::sync::Weak::ptr_eq(&self.owner, &Arc::downgrade(store))
    }
}

fn record_mut(
    records: &mut std::collections::VecDeque<DecisionRecord>,
    sequence: u64,
) -> Option<&mut DecisionRecord> {
    records
        .iter_mut()
        .find(|record| record.sequence == sequence)
}

fn bind_record(
    records: &mut std::collections::VecDeque<DecisionRecord>,
    sequence: u64,
    action: ActionId,
) -> bool {
    let Some(record) = record_mut(records, sequence) else {
        return false;
    };
    if record.eventual_outcome != DecisionOutcome::Pending {
        return false;
    }
    record.bind_action(action)
}

pub(super) fn with_elapsed(outcome: DecisionOutcome, elapsed_ms: u64) -> DecisionOutcome {
    match outcome {
        DecisionOutcome::Succeeded { bytes, .. } => {
            DecisionOutcome::Succeeded { bytes, elapsed_ms }
        }
        DecisionOutcome::HeadObserved {
            content_length,
            accept_ranges,
            ..
        } => DecisionOutcome::HeadObserved {
            content_length,
            accept_ranges,
            elapsed_ms,
        },
        DecisionOutcome::Failed { class, .. } => DecisionOutcome::Failed { class, elapsed_ms },
        DecisionOutcome::Cancelled { bytes, .. } => {
            DecisionOutcome::Cancelled { bytes, elapsed_ms }
        }
        other => other,
    }
}

pub(super) fn unix_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u128::from(u64::MAX)) as u64
}
