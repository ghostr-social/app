mod claim;
mod commands;
mod lifecycle;
mod publication;
mod retention;

use ghostr_engine::adaptive::{DecisionAction, DecisionOutcome, DecisionPrivacy, DecisionRecord};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard, Weak};

const HISTORY_CAPACITY: usize = 64;

pub(crate) struct DecisionResolution {
    pub action: DecisionAction,
    pub warp_action: Option<ghostr_engine::adaptive::RecordedWarpAction>,
    pub elapsed_ms: u64,
}

use publication::DecisionPublication;
pub(crate) use publication::{LegacyDecisionPublication, WarpDecisionPublication};

/// Exact correlation for one decision published by this log instance.
#[must_use = "a selected decision must be bound or resolved"]
pub(crate) struct DecisionToken {
    sequence: u64,
    owner: Weak<Mutex<DecisionStore>>,
    armed: bool,
}

pub(crate) struct RequestDecisionBinding<'a> {
    action: ghostr_engine::ActionId,
    request: &'a ghostr_engine::adaptive::ExecutedRequest,
    observed_at_ms: u64,
}

impl<'a> RequestDecisionBinding<'a> {
    pub(crate) const fn new(
        action: ghostr_engine::ActionId,
        request: &'a ghostr_engine::adaptive::ExecutedRequest,
        observed_at_ms: u64,
    ) -> Self {
        Self {
            action,
            request,
            observed_at_ms,
        }
    }
}

/// One admitted asynchronous decision that cannot be superseded while live.
#[must_use = "a claimed decision must reach a terminal outcome"]
pub(crate) struct DecisionClaim {
    sequence: u64,
    owner: Weak<Mutex<DecisionStore>>,
    started_at_ms: u64,
    armed: bool,
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
    claimed: HashSet<u64>,
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
                claimed: HashSet::new(),
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
    fn publish(&self, publication: DecisionPublication<'_>) -> Option<DecisionToken> {
        let mut store = self.lock();
        let superseded = {
            let store = &mut *store;
            retention::supersede_unbound(&mut store.records, &store.claimed)
        };
        if let Some(sequence) = superseded {
            store.completed.push_back(sequence);
        }
        let sequence = next_sequence(&mut store);
        let mut record = publication.capture(sequence, &self.privacy);
        let legacy_noop = record.warp_decision.is_none()
            && record.chosen_action.is_none()
            && record.resolve(DecisionOutcome::Succeeded {
                bytes: 0,
                elapsed_ms: 0,
            });
        let pending = record.eventual_outcome == DecisionOutcome::Pending;
        store.records.push_back(record);
        if !pending {
            store.completed.push_back(sequence);
        }
        trim(&mut store);
        debug_assert!(!legacy_noop || !pending);
        pending.then(|| DecisionToken::new(sequence, &self.store))
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

fn next_sequence(store: &mut DecisionStore) -> u64 {
    store.next_sequence = store
        .next_sequence
        .checked_add(1)
        .expect("decision sequence exhausted");
    store.next_sequence
}

fn trim(store: &mut DecisionStore) {
    retention::trim(&mut store.records, &mut store.completed);
}
