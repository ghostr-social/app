mod commands;
mod retention;

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
    completed: VecDeque<u64>,
}

impl Default for DecisionLog {
    fn default() -> Self {
        Self {
            store: Arc::new(Mutex::new(DecisionStore {
                next_sequence: 0,
                records: VecDeque::new(),
                actions: HashMap::new(),
                completed: VecDeque::new(),
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
        if let Some(sequence) = retention::supersede_unbound(&mut store.records) {
            store.completed.push_back(sequence);
        }
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
        let completed = record.chosen_action.is_none()
            && record.resolve(DecisionOutcome::Succeeded {
                bytes: 0,
                elapsed_ms: 0,
            });
        store.records.push_back(record);
        if completed {
            store.completed.push_back(sequence);
        }
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
        let (sequence, started_at_ms) = store.actions.get(&action.value()).copied()?;
        let elapsed_ms = observed_at_ms.saturating_sub(started_at_ms);
        let outcome = with_elapsed(outcome, elapsed_ms);
        let decision = retention::resolve(&mut store.records, sequence, outcome)?;
        store.actions.remove(&action.value());
        store.completed.push_back(sequence);
        trim(&mut store);
        Some(DecisionResolution {
            action: decision,
            elapsed_ms,
        })
    }

    fn resolve_latest(&self, outcome: DecisionOutcome) -> bool {
        let mut store = self.lock();
        let Some(sequence) = retention::resolve_latest(&mut store.records, outcome) else {
            return false;
        };
        store.completed.push_back(sequence);
        trim(&mut store);
        true
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

fn trim(store: &mut DecisionStore) {
    retention::trim(&mut store.records, &mut store.completed);
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
