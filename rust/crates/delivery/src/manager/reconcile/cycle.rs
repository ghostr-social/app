use super::PlanningStoreState;
use crate::manager::concurrency::RequestConcurrencyLimits;
use crate::manager::inflight::ActiveAction;
use crate::manager::plan::{planned_work_with_planner, PlanInputs, PlannedWork};
use crate::manager::DeliveryWorker;
use ghostr_engine::adaptive::StorageSnapshot;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::{ByteRange, PostId};
use ghostr_partial_store::partial_range_store::capacity::CapacitySnapshot;
use std::collections::{HashMap, HashSet};

pub(super) struct PlanningCycle {
    observed_at_ms: u64,
    limits: RequestConcurrencyLimits,
    capacity: CapacitySnapshot,
    pub(super) stored: PlanningStoreState,
    independent_sources: HashMap<PostId, HashSet<String>>,
    in_flight: Vec<ActiveAction>,
    active_head_probes: Vec<TransferIdentity>,
    demanded: HashMap<PostId, ByteRange>,
}

impl DeliveryWorker {
    pub(super) fn reconcile_request_surfaces(&mut self, limits: RequestConcurrencyLimits) {
        self.sync_request_gate(limits);
        self.segmented
            .reconcile(crate::segmented::scheduler::ReconcileInput {
                requests: self.ctx.requests.clone(),
                events: self.ctx.events.clone(),
                connection_limit: limits.segmented_compatibility(),
                progressive_active: self.downloads.len(),
            });
    }

    pub(super) async fn prepare_planning_cycle(
        &mut self,
        observed_at_ms: u64,
        limits: RequestConcurrencyLimits,
    ) -> PlanningCycle {
        let capacity = self.ctx.store.capacity_snapshot().await;
        let window = self.collection_window(&capacity);
        let timeline_window = self.state.timeline_window_posts();
        let timeline_posts: HashSet<_> = timeline_window.iter().cloned().collect();
        let stored = self.collect_stored(&window, &timeline_posts).await;
        self.state
            .replace_transformed_posts(stored.transformed.clone());
        self.state.prune_player_preparations(&stored.revisions);
        let independent_sources = self.independent_objects.current(&stored.revisions);
        self.reconcile_timelines(&timeline_window, &stored.snapshots);
        self.state.reconcile_fast_start_evidence(&stored.snapshots);
        self.reconcile_probe_bodies();
        let in_flight = self.downloads.actions();
        let active_head_probes = self.probes.active_identities();
        let demanded = self.resolve_gateway_demands(&stored.present);
        PlanningCycle {
            observed_at_ms,
            limits,
            capacity,
            stored,
            independent_sources,
            in_flight,
            active_head_probes,
            demanded,
        }
    }

    pub(super) fn plan_cycle(&mut self, cycle: &PlanningCycle) -> PlannedWork {
        let inputs = PlanInputs {
            stats: self.keeper.stats(),
            retry: &self.retry,
            present: &cycle.stored.present,
            finalized: &cycle.stored.finalized,
            stored_totals: &cycle.stored.totals,
            continuation_sources: &cycle.stored.continuation_sources,
            revisions: &cycle.stored.revisions,
            independent_sources: &cycle.independent_sources,
            completed_head_probes: self.probes.completed_posts(),
            in_flight: &cycle.in_flight,
            active_head_probes: &cycle.active_head_probes,
            storage: StorageSnapshot::new(
                cycle.capacity.limit_bytes(),
                cycle.capacity.used_bytes(),
            ),
            connection_capacity: self
                .concurrency_limit()
                .min(self.progressive_capacity())
                .max(1),
            connection_ceiling: cycle.limits.global(),
            per_authority_request_limit: cycle.limits.per_authority(),
            packet_loss_bps: self.ctx.network.profile().packet_loss_bps,
            measured_network_bytes_per_second: self
                .keeper
                .network_load_bytes_per_second(cycle.observed_at_ms),
            capacity_revision: cycle.capacity.revision().value(),
            observed_at_ms: cycle.observed_at_ms,
            demanded: &cycle.demanded,
        };
        planned_work_with_planner(
            &mut self.state,
            inputs,
            &mut self.warp_planner,
            self.qoe.watch_model(),
        )
    }

    pub(super) fn finish_reconcile(&mut self) {
        self.keeper.schedule_save(&self.ctx.events);
        self.reliability.observe(
            self.state.catalog().reliability_revision(),
            &self.ctx.events,
        );
    }
}
