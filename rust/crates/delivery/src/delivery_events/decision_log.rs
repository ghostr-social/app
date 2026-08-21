mod commands;
mod lifecycle;
mod retention;

use ghostr_engine::adaptive::{
    AllocationPlan, DecisionAction, DecisionModelInput, DecisionOutcome, DecisionPrivacy,
    DecisionRecord, DecisionRecordInput, PlayabilitySnapshot, ShadowPrices,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard, Weak};

const HISTORY_CAPACITY: usize = 64;

pub(crate) struct DecisionResolution {
    pub action: DecisionAction,
    pub elapsed_ms: u64,
}

pub(crate) struct LegacyDecisionPublication<'a> {
    pub snapshot: &'a PlayabilitySnapshot,
    pub plan: &'a AllocationPlan,
    pub prices: ShadowPrices,
    pub models: &'a [DecisionModelInput],
}

/// Exact correlation for one decision published by this log instance.
#[must_use = "a selected decision must be bound or resolved"]
pub(crate) struct DecisionToken {
    sequence: u64,
    owner: Weak<Mutex<DecisionStore>>,
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
    actions: HashMap<ghostr_engine::ActionId, ActionBinding>,
    completed: VecDeque<u64>,
}

#[derive(Clone, Copy)]
struct ActionBinding {
    sequence: u64,
    started_at_ms: u64,
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
    fn publish(&self, publication: LegacyDecisionPublication<'_>) -> Option<DecisionToken> {
        let mut store = self.lock();
        if let Some(sequence) = retention::supersede_unbound(&mut store.records) {
            store.completed.push_back(sequence);
        }
        store.next_sequence = store
            .next_sequence
            .checked_add(1)
            .expect("decision sequence exhausted");
        let sequence = store.next_sequence;
        let mut record = DecisionRecord::capture(DecisionRecordInput {
            sequence,
            snapshot: publication.snapshot,
            allocation: publication.plan,
            shadow_prices: publication.prices,
            models: publication.models,
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
        (!completed).then(|| DecisionToken::new(sequence, &self.store))
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
