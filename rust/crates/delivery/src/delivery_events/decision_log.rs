mod commands;

use ghostr_engine::adaptive::{
    AllocationPlan, DecisionAction, DecisionModelInput, DecisionOutcome, DecisionPrivacy,
    DecisionRecord, DecisionRecordInput, PlayabilitySnapshot, ShadowPrices,
};
use ghostr_engine::ActionId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard};

const HISTORY_CAPACITY: usize = 64;

pub(crate) struct DecisionResolution {
    pub action: DecisionAction,
    pub elapsed_ms: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DecisionHistorySnapshot {
    pub records: Vec<DecisionRecord>,
}

#[derive(Clone)]
pub(super) struct DecisionLog {
    store: Arc<Mutex<DecisionStore>>,
    privacy: Arc<DecisionPrivacy>,
}

struct DecisionStore {
    next_sequence: u64,
    records: VecDeque<DecisionRecord>,
    actions: HashMap<u64, (u64, u64)>,
}

impl Default for DecisionLog {
    fn default() -> Self {
        Self {
            store: Arc::new(Mutex::new(DecisionStore {
                next_sequence: 0,
                records: VecDeque::new(),
                actions: HashMap::new(),
            })),
            privacy: Arc::new(DecisionPrivacy::from_key(rand::random())),
        }
    }
}

impl std::fmt::Debug for DecisionLog {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DecisionLog")
            .finish_non_exhaustive()
    }
}

impl DecisionLog {
    fn publish(
        &self,
        snapshot: &PlayabilitySnapshot,
        plan: &AllocationPlan,
        prices: ShadowPrices,
        models: &[DecisionModelInput],
    ) -> u64 {
        let mut store = self.lock();
        supersede_unbound(&mut store.records);
        store.next_sequence = store.next_sequence.saturating_add(1);
        let sequence = store.next_sequence;
        let mut record = DecisionRecord::capture(DecisionRecordInput {
            sequence,
            snapshot,
            allocation: plan,
            shadow_prices: prices,
            models,
            privacy: &self.privacy,
        });
        if record.chosen_action.is_none() {
            record.resolve(DecisionOutcome::Succeeded {
                bytes: 0,
                elapsed_ms: 0,
            });
        }
        store.records.push_back(record);
        trim(&mut store);
        sequence
    }

    fn bind_latest(&self, action: ActionId, observed_at_ms: u64) -> bool {
        let mut store = self.lock();
        let Some(record) = store.records.iter_mut().rev().find(|record| {
            record.eventual_outcome == DecisionOutcome::Pending && record.chosen_action_id.is_none()
        }) else {
            return false;
        };
        let sequence = record.sequence;
        if !record.bind_action(action) {
            return false;
        }
        store
            .actions
            .insert(action.value(), (sequence, observed_at_ms));
        true
    }

    fn resolve(
        &self,
        action: ActionId,
        outcome: DecisionOutcome,
        observed_at_ms: u64,
    ) -> Option<DecisionResolution> {
        let mut store = self.lock();
        let (sequence, started_at_ms) = store.actions.remove(&action.value())?;
        let elapsed_ms = observed_at_ms.saturating_sub(started_at_ms);
        let outcome = with_elapsed(outcome, elapsed_ms);
        let record = store
            .records
            .iter_mut()
            .find(|record| record.sequence == sequence)?;
        let action = record.chosen_action.clone()?;
        record
            .resolve(outcome)
            .then_some(DecisionResolution { action, elapsed_ms })
    }

    fn resolve_latest(&self, outcome: DecisionOutcome) -> bool {
        let mut store = self.lock();
        store
            .records
            .iter_mut()
            .rev()
            .find(|record| {
                record.eventual_outcome == DecisionOutcome::Pending
                    && record.chosen_action_id.is_none()
            })
            .is_some_and(|record| record.resolve(outcome))
    }

    pub(super) fn snapshot(&self) -> DecisionHistorySnapshot {
        DecisionHistorySnapshot {
            records: self.lock().records.iter().cloned().collect(),
        }
    }

    fn lock(&self) -> MutexGuard<'_, DecisionStore> {
        self.store
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn supersede_unbound(records: &mut VecDeque<DecisionRecord>) {
    if let Some(record) = records.iter_mut().rev().find(|record| {
        record.eventual_outcome == DecisionOutcome::Pending && record.chosen_action_id.is_none()
    }) {
        record.resolve(DecisionOutcome::Superseded);
    }
}

fn trim(store: &mut DecisionStore) {
    while store.records.len() > HISTORY_CAPACITY {
        if let Some(removed) = store.records.pop_front() {
            store
                .actions
                .retain(|_, (sequence, _)| *sequence != removed.sequence);
        }
    }
}

fn with_elapsed(outcome: DecisionOutcome, elapsed_ms: u64) -> DecisionOutcome {
    match outcome {
        DecisionOutcome::Succeeded { bytes, .. } => {
            DecisionOutcome::Succeeded { bytes, elapsed_ms }
        }
        DecisionOutcome::Failed { class, .. } => DecisionOutcome::Failed { class, elapsed_ms },
        DecisionOutcome::Cancelled { bytes, .. } => {
            DecisionOutcome::Cancelled { bytes, elapsed_ms }
        }
        other => other,
    }
}
