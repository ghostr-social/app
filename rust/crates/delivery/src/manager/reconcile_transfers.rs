//! Reconciles the pure policy's ordered work with live transfer workers.

use crate::manager::concurrency::{connection_ceiling, planned_capacity};
use crate::manager::plan::{PlannedTransfer, PlannedWork};
use crate::manager::DeliveryWorker;
use crate::mutable_priority_queue::ForegroundSlots;
use std::collections::HashSet;

impl DeliveryWorker {
    pub(super) fn reconcile_transfers(&mut self, planned: PlannedWork) {
        let emergency = planned.emergency;
        let retained_posts: HashSet<_> = planned
            .plan
            .retained
            .iter()
            .map(|work| work.post.clone())
            .collect();
        let capacity = planned_capacity(
            self.concurrency_limit(),
            self.connection_ceiling(),
            &planned.transfers,
            &retained_posts,
        );
        let priority: Vec<_> = planned
            .transfers
            .iter()
            .map(|transfer| transfer.request.chunk.clone())
            .collect();
        self.downloads.reconcile_with_commitments(
            &planned.transfers,
            capacity.total,
            &planned.retained,
        );
        self.queue.replace(planned.transfers);
        self.preempt_for_current(&priority, capacity.total);
        self.grant_planned(capacity.total, capacity.foreground_goal);
        if !emergency {
            self.grant_origin_exploration();
        }
    }

    fn preempt_for_current(&mut self, priority: &[ghostr_engine::ChunkId], capacity: usize) {
        if let Some(current) = self.state.focus().current() {
            self.downloads
                .preempt_for_current(current, priority, capacity);
        }
    }

    fn grant_planned(&mut self, capacity: usize, foreground_goal: usize) {
        while self.downloads.len() < capacity {
            let active_hosts = self.downloads.active_hosts();
            let foreground = ForegroundSlots::new(self.downloads.foreground_len(), foreground_goal);
            let Some(transfer) = self.queue.pop_for_hosts(&active_hosts, foreground) else {
                return;
            };
            self.grant(transfer);
        }
    }

    fn grant_origin_exploration(&mut self) {
        let exploration_limit = self
            .concurrency_limit()
            .saturating_add(1)
            .min(self.state.concurrency());
        if self.downloads.len() >= exploration_limit {
            return;
        }
        let active_hosts = self.downloads.active_hosts();
        if let Some(transfer) = self.queue.pop_for_idle_host(&active_hosts) {
            self.grant(transfer);
        }
    }

    fn grant(&mut self, transfer: PlannedTransfer) {
        let chunk = &transfer.request.chunk;
        if self.downloads.contains(chunk)
            || self.retry.is_cooling(&chunk.post)
            || self.pressure.is_parked()
        {
            return;
        }
        self.downloads.start(self.ctx.clone(), transfer);
    }

    pub(super) fn connection_ceiling(&self) -> usize {
        connection_ceiling(
            self.state.concurrency(),
            self.ctx.network.profile().max_connections_per_host,
        )
    }
}
