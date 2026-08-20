use super::{CommandReceiver, DeliveryHandle};
use crate::startup_certificate::StartupCertificate;
use ghostr_engine::adaptive::AllocationPlan;
use ghostr_engine::PostId;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::Notify;

const HISTORY_CAPACITY: usize = 512;

#[derive(Clone, Debug, PartialEq)]
pub struct PlanEvidence {
    pub revision: u64,
    pub observed_at_ms: u64,
    pub current: Option<PostId>,
    pub plan: AllocationPlan,
    pub startups: Vec<StartupCertificate>,
}

#[derive(Clone, Debug, Default)]
pub(super) struct PlanEvidenceHistory {
    store: Arc<Mutex<PlanEvidenceStore>>,
    changed: Arc<Notify>,
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
        self.publish_focused(observed_at_ms, None, plan, Vec::new());
    }

    pub(super) fn publish_focused(
        &self,
        observed_at_ms: u64,
        current: Option<PostId>,
        plan: AllocationPlan,
        startups: Vec<StartupCertificate>,
    ) {
        let mut store = self.lock();
        store.next_revision = store.next_revision.saturating_add(1);
        let revision = store.next_revision;
        store.history.push_back(PlanEvidence {
            revision,
            observed_at_ms,
            current,
            plan,
            startups,
        });
        if store.history.len() > HISTORY_CAPACITY {
            store.history.pop_front();
        }
        drop(store);
        self.changed.notify_waiters();
    }

    pub(super) fn latest(&self) -> Option<PlanEvidence> {
        self.lock().history.back().cloned()
    }

    pub(super) fn notifier(&self) -> Arc<Notify> {
        self.changed.clone()
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
    pub fn publish_plan(&mut self, observed_at_ms: u64, plan: AllocationPlan) {
        self.plans.publish(observed_at_ms, plan);
    }

    pub fn publish_focused_plan(
        &mut self,
        observed_at_ms: u64,
        current: Option<PostId>,
        plan: AllocationPlan,
    ) {
        self.plans
            .publish_focused(observed_at_ms, current, plan, Vec::new());
    }

    pub fn publish_focused_plan_with_startup(
        &mut self,
        observed_at_ms: u64,
        current: Option<PostId>,
        plan: AllocationPlan,
        startup: Option<StartupCertificate>,
    ) {
        self.publish_focused_plan_with_startups(
            observed_at_ms,
            current,
            plan,
            startup.into_iter().collect(),
        );
    }

    pub fn publish_focused_plan_with_startups(
        &mut self,
        observed_at_ms: u64,
        current: Option<PostId>,
        plan: AllocationPlan,
        startups: Vec<StartupCertificate>,
    ) {
        self.plans
            .publish_focused(observed_at_ms, current, plan, startups);
    }
}
