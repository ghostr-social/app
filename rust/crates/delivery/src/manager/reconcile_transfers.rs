//! Reconciles the pure policy's ordered work with live transfer workers.

mod grant;

use crate::delivery_events::DecisionToken;
use crate::manager::concurrency::{planned_capacity, RequestConcurrencyLimits};
use crate::manager::plan::PlannedWork;
use crate::manager::reconcile_warp::{self, WarpDirective};
use crate::manager::selected_commit::SelectedCommit;
use crate::manager::DeliveryWorker;
use crate::mutable_priority_queue::ForegroundSlots;

struct SelectedGrant<'a> {
    directive: &'a WarpDirective,
    decision: &'a mut Option<DecisionToken>,
    commit: &'a mut Option<SelectedCommit>,
}

impl DeliveryWorker {
    pub(super) async fn reconcile_transfers(
        &mut self,
        planned: PlannedWork,
        mut decision: Option<DecisionToken>,
    ) {
        let execution = reconcile_warp::execution(planned);
        let mut commit = SelectedCommit::optional(execution.selected);
        self.apply_warp_directive(&execution.directive, &mut decision, &mut commit)
            .await;
        let capacity = planned_capacity(
            self.concurrency_limit(),
            self.connection_ceiling(),
            &execution.transfers,
            &execution.retained_posts,
        )
        .with_selected_hedge(
            self.downloads.len(),
            self.connection_ceiling(),
            matches!(execution.directive, WarpDirective::Hedge { .. }),
        );
        let total = capacity.total.min(self.progressive_capacity());
        let priority: Vec<_> = execution
            .transfers
            .iter()
            .map(|transfer| transfer.request.chunk.clone())
            .collect();
        self.downloads
            .reconcile_with_commitments(&execution.transfers, total, &execution.retained);
        self.queue.replace(execution.transfers);
        self.preempt_for_current(&priority, total);
        let selected = SelectedGrant {
            directive: &execution.directive,
            decision: &mut decision,
            commit: &mut commit,
        };
        self.grant_planned(total, capacity.foreground_goal.min(total), selected)
            .await;
        if !execution.emergency {
            self.grant_origin_exploration().await;
        }
    }

    fn preempt_for_current(&mut self, priority: &[ghostr_engine::ChunkId], capacity: usize) {
        if let Some(current) = self.state.focus().current() {
            self.downloads
                .preempt_for_current(current, priority, capacity);
        }
    }

    async fn grant_planned(
        &mut self,
        capacity: usize,
        foreground_goal: usize,
        selected: SelectedGrant<'_>,
    ) {
        while self.downloads.len() < capacity {
            let active_hosts = self.downloads.active_hosts();
            let foreground = ForegroundSlots::new(self.downloads.foreground_len(), foreground_goal);
            let Some(transfer) = self.queue.pop_for_hosts(&active_hosts, foreground) else {
                return;
            };
            let alternate = transfer.id();
            if let Some(action) = self
                .grant(transfer, selected.decision, selected.commit)
                .await
            {
                self.link_selected_hedge(selected.directive, &alternate, action);
            }
        }
    }

    async fn grant_origin_exploration(&mut self) {
        let exploration_limit = self
            .concurrency_limit()
            .saturating_add(1)
            .min(self.state.concurrency())
            .min(self.progressive_capacity());
        if self.downloads.len() >= exploration_limit {
            return;
        }
        let active_hosts = self.downloads.active_hosts();
        if let Some(transfer) = self.queue.pop_for_idle_host(&active_hosts) {
            let _ = self.grant(transfer, &mut None, &mut None).await;
        }
    }

    pub(super) fn connection_ceiling(&self) -> usize {
        self.request_limits().global()
    }

    pub(super) fn request_limits(&self) -> RequestConcurrencyLimits {
        RequestConcurrencyLimits::resolve(
            self.state.concurrency(),
            self.max_requests_per_authority,
            self.ctx.network.profile().max_connections_per_host,
        )
    }

    pub(super) fn progressive_capacity(&self) -> usize {
        self.connection_ceiling()
            .saturating_sub(self.segmented.active_len())
    }
}
