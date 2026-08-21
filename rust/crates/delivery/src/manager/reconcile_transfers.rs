//! Reconciles the pure policy's ordered work with live transfer workers.

use crate::manager::concurrency::{planned_capacity, RequestConcurrencyLimits};
use crate::manager::plan::{PlannedTransfer, PlannedWork};
use crate::manager::reconcile_warp::{self, WarpDirective};
use crate::manager::DeliveryWorker;
use crate::manager::{origin_admission, time};
use crate::mutable_priority_queue::ForegroundSlots;

impl DeliveryWorker {
    pub(super) async fn reconcile_transfers(&mut self, planned: PlannedWork) {
        let execution = reconcile_warp::execution(planned);
        self.apply_warp_directive(&execution.directive);
        let capacity = planned_capacity(
            self.concurrency_limit(),
            self.connection_ceiling(),
            &execution.transfers,
            &execution.retained_posts,
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
        self.grant_planned(
            total,
            capacity.foreground_goal.min(total),
            &execution.directive,
        )
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
        directive: &WarpDirective,
    ) {
        while self.downloads.len() < capacity {
            let active_hosts = self.downloads.active_hosts();
            let foreground = ForegroundSlots::new(self.downloads.foreground_len(), foreground_goal);
            let Some(transfer) = self.queue.pop_for_hosts(&active_hosts, foreground) else {
                return;
            };
            let alternate = transfer.id();
            if let Some(action) = self.grant(transfer).await {
                self.link_selected_hedge(directive, &alternate, action);
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
            let _ = self.grant(transfer).await;
        }
    }

    async fn grant(&mut self, transfer: PlannedTransfer) -> Option<ghostr_engine::ActionId> {
        let post = &transfer.request.chunk.post;
        if self.downloads.contains_transfer(&transfer)
            || self.retry.is_cooling(post)
            || self.pressure.is_parked()
        {
            return None;
        }
        let (transfer, observed_at_ms) = self.admit_origin(transfer)?;
        let post = transfer.request.chunk.post.clone();
        match self.downloads.start(self.ctx.clone(), transfer).await {
            Ok(action) => {
                self.commands.bind_latest_decision(action, observed_at_ms);
                Some(action)
            }
            Err(error) => {
                self.reject_grant(&post, &error);
                None
            }
        }
    }

    fn admit_origin(&mut self, transfer: PlannedTransfer) -> Option<(PlannedTransfer, u64)> {
        let observed_at_ms = time::unix_time_ms();
        let authority = ghostr_engine::RequestAuthority::from_url(&transfer.url)?;
        let concurrency = origin_concurrency(&self.ctx.network, &authority);
        let query = origin_admission::query(&transfer, observed_at_ms, concurrency);
        let mode = origin_admission::mode(&transfer);
        let admission =
            self.keeper
                .stats_mut()
                .origin_model_mut()
                .claim(&query, observed_at_ms, mode);
        origin_admission::apply(transfer, admission).map(|value| (value, observed_at_ms))
    }

    fn reject_grant(&mut self, post: &ghostr_engine::PostId, error: &anyhow::Error) {
        self.commands
            .resolve_latest_decision(ghostr_engine::adaptive::DecisionOutcome::Failed {
                class: format!("{:?}", crate::manager::failure::classify(error)),
                elapsed_ms: 0,
            });
        if !self.absorb_store_pressure(post, error) {
            log::warn!("Could not reserve a video action: {error:#}");
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

fn origin_concurrency(
    network: &crate::debug::network::NetworkThrottle,
    authority: &ghostr_engine::RequestAuthority,
) -> usize {
    network
        .active_connections()
        .into_iter()
        .find(|(active, _)| active == authority.as_str())
        .map_or(1, |(_, count)| count.saturating_add(1))
}
