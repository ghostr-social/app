use super::events::{
    AdaptationMetricEvent, BudgetMetricEvent, IntegrityMetricEvent, PlaybackMetricEvent,
    PresentationMetricEvent, ReadinessMetricEvent, SemanticMetricEvent, SemanticMetricRollup,
    TransferMetricEvent,
};
use super::{EvaluationSnapshot, EvaluationTracker};
use ghostr_engine::PostId;
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Clone, Default)]
pub(crate) struct EvaluationLedger {
    tracker: Arc<Mutex<EvaluationTracker>>,
}

impl core::fmt::Debug for EvaluationLedger {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("EvaluationLedger")
            .finish_non_exhaustive()
    }
}

impl EvaluationLedger {
    pub(crate) fn focus(&self, post: Option<PostId>, at_ms: u64) {
        let mut tracker = self.lock();
        match post {
            Some(post) => tracker.focus(post, at_ms),
            None => tracker.finish(at_ms),
        }
    }

    pub(crate) fn playback(&self, event: &PlaybackMetricEvent) {
        self.lock().playback(event);
    }

    pub(crate) fn present(&self, event: &PresentationMetricEvent) {
        self.lock().present(event);
    }

    pub(crate) fn transfer(&self, event: TransferMetricEvent) {
        self.lock().transfer(event);
    }

    pub(crate) fn budget(&self, event: BudgetMetricEvent) {
        self.lock().budget(event);
    }

    pub(crate) fn readiness(&self, event: ReadinessMetricEvent) {
        self.lock().readiness(event);
    }

    pub(crate) fn adaptation(&self, event: &AdaptationMetricEvent) {
        self.lock().adaptation(event);
    }

    pub(crate) fn semantic(&self, event: SemanticMetricEvent) {
        self.lock().semantic(event);
    }

    pub(crate) fn semantic_rollup(&self, event: SemanticMetricRollup) {
        self.lock().semantic_rollup(event);
    }

    pub(crate) fn integrity(&self, event: IntegrityMetricEvent) {
        self.lock().integrity(event);
    }

    pub(crate) fn snapshot(&self) -> EvaluationSnapshot {
        self.lock().snapshot()
    }

    fn lock(&self) -> MutexGuard<'_, EvaluationTracker> {
        self.tracker
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
