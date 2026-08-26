use super::*;

impl InFlightChunks {
    /// Retains planned IO, then reserves slots for higher-priority work.
    pub(crate) fn reconcile(&mut self, planned: &[PlannedTransfer], capacity: usize) {
        self.reconcile_with_commitments(planned, capacity, &HashSet::new());
    }
}
