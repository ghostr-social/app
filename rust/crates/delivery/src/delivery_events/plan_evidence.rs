use ghostr_engine::adaptive::AllocationPlan;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};

const HISTORY_CAPACITY: usize = 512;

#[derive(Clone, Debug, PartialEq)]
pub struct PlanEvidence {
    pub revision: u64,
    pub observed_at_ms: u64,
    pub plan: AllocationPlan,
}

#[derive(Clone, Debug, Default)]
pub(super) struct PlanEvidenceHistory {
    store: Arc<Mutex<PlanEvidenceStore>>,
}

#[derive(Debug, Default)]
struct PlanEvidenceStore {
    next_revision: u64,
    history: VecDeque<PlanEvidence>,
}

impl PlanEvidenceHistory {
    pub(super) fn snapshot(&self) -> Vec<PlanEvidence> {
        self.lock().history.iter().cloned().collect()
    }

    pub(super) fn publish(&self, observed_at_ms: u64, plan: AllocationPlan) {
        let mut store = self.lock();
        store.next_revision = store.next_revision.saturating_add(1);
        let revision = store.next_revision;
        store.history.push_back(PlanEvidence {
            revision,
            observed_at_ms,
            plan,
        });
        if store.history.len() > HISTORY_CAPACITY {
            store.history.pop_front();
        }
    }

    fn lock(&self) -> MutexGuard<'_, PlanEvidenceStore> {
        self.store
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
