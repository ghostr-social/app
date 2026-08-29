use super::{CommandReceiver, DeliveryHandle};
use crate::startup_certificate::StartupCertificate;
use ghostr_engine::adaptive::AllocationPlan;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::Notify;

const HISTORY_CAPACITY: usize = 512;

mod model;
pub use model::PlanEvidence;
pub(crate) use model::PlanPublicationContext;

#[derive(Clone, Debug, Default)]
pub(super) struct PlanEvidenceHistory {
    store: Arc<Mutex<PlanEvidenceStore>>,
    changed: Arc<Notify>,
}

pub(super) struct PlanEvidencePage {
    pub oldest_retained_revision: u64,
    pub latest_retained_revision: u64,
    pub cursor_truncated: bool,
    pub has_more: bool,
    pub records: Vec<PlanEvidence>,
}

#[derive(Debug, Default)]
struct PlanEvidenceStore {
    next_revision: u64,
    history: VecDeque<PlanEvidence>,
}

impl PlanEvidenceHistory {
    fn snapshot(&self) -> Vec<PlanEvidence> {
        self.lock().history.iter().cloned().collect()
    }

    pub(super) fn page(&self, after_revision: u64, limit: usize) -> PlanEvidencePage {
        let store = self.lock();
        let oldest = store.history.front().map_or(0, |plan| plan.revision);
        let latest = store.history.back().map_or(0, |plan| plan.revision);
        let truncated = after_revision.saturating_add(1) < oldest;
        let cursor = if truncated {
            oldest.saturating_sub(1)
        } else {
            after_revision
        };
        let mut remaining = store.history.iter().filter(|plan| plan.revision > cursor);
        let records = remaining.by_ref().take(limit).cloned().collect();
        PlanEvidencePage {
            oldest_retained_revision: oldest,
            latest_retained_revision: latest,
            cursor_truncated: truncated,
            has_more: remaining.next().is_some(),
            records,
        }
    }

    fn publish_focused(
        &self,
        context: PlanPublicationContext,
        plan: AllocationPlan,
        startups: Vec<StartupCertificate>,
    ) {
        let mut store = self.lock();
        store.next_revision = store.next_revision.saturating_add(1);
        let revision = store.next_revision;
        store.history.push_back(PlanEvidence {
            revision,
            decision_sequence: context.decision_sequence,
            observed_at_ms: context.observed_at_ms,
            current: context.current,
            focus_generation: context.focus_generation,
            focus_covers_from: context.focus_covers_from,
            network_status_generation: context.network_status.generation(),
            network_class: context.network_status.network_class(),
            network_profile_generation: context.network_profile_generation,
            player_preparations: context.player_preparations,
            plan,
            startups,
        });
        if store.history.len() > HISTORY_CAPACITY {
            store.history.pop_front();
        }
        drop(store);
        self.changed.notify_waiters();
    }

    fn latest(&self) -> Option<PlanEvidence> {
        self.lock().history.back().cloned()
    }

    fn notifier(&self) -> Arc<Notify> {
        std::sync::Arc::clone(&self.changed)
    }

    fn lock(&self) -> MutexGuard<'_, PlanEvidenceStore> {
        self.store
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl DeliveryHandle {
    pub fn plan_history(&self) -> Vec<PlanEvidence> {
        self.plans.snapshot()
    }

    pub fn latest_plan(&self) -> Option<PlanEvidence> {
        self.plans.latest()
    }

    pub fn plan_notifier(&self) -> Arc<Notify> {
        self.plans.notifier()
    }
}

impl CommandReceiver {
    pub(crate) fn publish_causal_plan_with_startups(
        &self,
        context: PlanPublicationContext,
        plan: AllocationPlan,
        startups: Vec<StartupCertificate>,
    ) {
        self.plans.publish_focused(context, plan, startups);
    }
}

#[cfg(any(test, feature = "test"))]
#[path = "plan_evidence/test_support.rs"]
mod test_support;
