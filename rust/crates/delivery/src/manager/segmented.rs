use crate::manager::{time, DeliveryWorker};
use crate::segmented::scheduler::SegmentedDone;

impl DeliveryWorker {
    pub(crate) fn finish_segmented(&mut self, done: SegmentedDone) {
        let Some(finish) = self.segmented.finish(done) else {
            return;
        };
        let observed_at_ms = time::unix_time_ms();
        let actual_network_bytes = finish
            .actual_resources
            .map_or(0, |actual| actual.network_bytes);
        self.warp_planner.reconcile_network_reservation(
            finish.resources.reserved_network_bytes(),
            actual_network_bytes,
            observed_at_ms,
        );
        if let Some(observation) = finish.observation {
            self.keeper.note_hls(observation);
        }
        match finish.actual_resources {
            Some(actual) => self.commands.resolve_decision_with_resources(
                finish.action,
                finish.outcome,
                actual,
                observed_at_ms,
            ),
            None => self
                .commands
                .resolve_decision(finish.action, finish.outcome, observed_at_ms),
        };
        self.request_immediate_replan();
    }
}
