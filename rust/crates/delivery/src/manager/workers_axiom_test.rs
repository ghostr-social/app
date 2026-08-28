use super::*;

impl DownloadWorkers {
    pub(in super::super) fn insert_test_action(
        &mut self,
        registration: crate::manager::inflight::ActionRegistration<'_>,
    ) {
        self.active.insert_action(registration);
    }

    pub(in super::super) fn insert_test_attempt(&mut self, attempt: &ChunkAttempt) {
        let request = ghostr_engine::scheduling::RangeRequest {
            chunk: attempt.chunk.clone(),
            authority: ghostr_engine::adaptive::PreemptionAuthority::Transition,
            score: 1.0,
            contiguous_depth_bytes: 0,
        };
        let (handle, _token) = crate::chunk::cancel::cancel_pair();
        self.active
            .insert(attempt, request, "fixture.example".into(), 0, handle);
    }
    pub(crate) fn reconcile(&mut self, planned: &[PlannedTransfer], capacity: usize) {
        self.reconcile_with_commitments(planned, capacity, &HashSet::new());
    }
}
