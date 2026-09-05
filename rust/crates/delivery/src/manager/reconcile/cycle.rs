use super::PlanningStoreState;
use crate::manager::concurrency::RequestConcurrencyLimits;
use crate::manager::inflight::ActiveAction;
use crate::manager::plan::{planned_work_with_planner, PlanInputs, PlannedWork};
use crate::manager::resource_control::ResourceEnvironment;
use crate::manager::DeliveryWorker;
use ghostr_engine::adaptive::{
    ResourceObservation, ShadowPriceController, StorageSnapshot, WholeBodyExhaustion,
};
use ghostr_engine::host_stats::OPTIMISTIC_THROUGHPUT_BPS;
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
    whole_body_exhaustions: HashMap<TransferIdentity, WholeBodyExhaustion>,
    in_flight: Vec<ActiveAction>,
    active_head_probes: Vec<TransferIdentity>,
    hls_candidates: Vec<ghostr_engine::adaptive::HlsCandidateSnapshot>,
    active_hls_sources: Vec<String>,
    segmented_storage_available_bytes: u64,
    segmented_storage_used_bytes: u64,
    segmented_storage_capacity_bytes: u64,
    demanded: HashMap<PostId, ByteRange>,
}

impl DeliveryWorker {
    pub(super) fn reconcile_request_surfaces(&self, limits: RequestConcurrencyLimits) {
        self.sync_request_gate(limits);
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
        self.reconcile_stored_cycle(&stored, &timeline_window);
        let independent_sources = self.independent_objects.current(self.state.catalog());
        let whole_body_exhaustions = self.whole_body_limits.current(self.state.catalog());
        let in_flight = self.downloads.actions();
        let active_head_probes = self.probes.active_identities();
        let navigation = self.state.navigation(observed_at_ms);
        let hls_candidates = self.segmented.planning_candidates(navigation, &self.state);
        let segmented_storage_available_bytes = self.segmented.available_bytes();
        let segmented_storage_used_bytes = self.segmented.used_bytes();
        let segmented_storage_capacity_bytes = crate::segmented::SegmentedCache::capacity_bytes();
        let active_hls_sources = self
            .segmented
            .active_sources()
            .into_iter()
            .map(str::to_owned)
            .collect();
        let demanded = self.resolve_gateway_demands(&stored.present);
        PlanningCycle {
            observed_at_ms,
            limits,
            capacity,
            stored,
            independent_sources,
            whole_body_exhaustions,
            in_flight,
            active_head_probes,
            hls_candidates,
            active_hls_sources,
            segmented_storage_available_bytes,
            segmented_storage_used_bytes,
            segmented_storage_capacity_bytes,
            demanded,
        }
    }

    fn reconcile_stored_cycle(&mut self, stored: &PlanningStoreState, timeline_window: &[PostId]) {
        self.downloads.cancel_covered_without_body(
            &stored.present,
            &stored.transformed,
            self.state.catalog(),
        );
        self.state
            .replace_transformed_posts(stored.transformed.clone());
        self.state.prune_player_preparations(&stored.revisions);
        self.reconcile_timelines(timeline_window, &stored.snapshots);
        self.state.reconcile_fast_start_evidence(&stored.snapshots);
        self.reconcile_probe_bodies();
    }

    pub(super) fn plan_cycle(&mut self, cycle: &PlanningCycle) -> PlannedWork {
        let environment = self.resource_environment(cycle);
        let completed_head_probes = self
            .probes
            .current_completed_identities(self.state.catalog());
        let unavailable_head_probes = self
            .probes
            .current_unavailable_identities(self.state.catalog());
        let inputs = PlanInputs {
            stats: self.keeper.stats(),
            retry: &self.retry,
            present: &cycle.stored.present,
            finalized: &cycle.stored.finalized,
            stored_totals: &cycle.stored.totals,
            continuation_sources: &cycle.stored.continuation_sources,
            revisions: &cycle.stored.revisions,
            independent_sources: &cycle.independent_sources,
            whole_body_exhaustions: &cycle.whole_body_exhaustions,
            completed_head_probes: &completed_head_probes,
            unavailable_head_probes: &unavailable_head_probes,
            in_flight: &cycle.in_flight,
            active_head_probes: &cycle.active_head_probes,
            hls_candidates: &cycle.hls_candidates,
            active_hls_sources: &cycle.active_hls_sources,
            segmented_storage_available_bytes: cycle.segmented_storage_available_bytes,
            storage: StorageSnapshot::new(
                cycle.capacity.limit_bytes(),
                cycle.capacity.used_bytes(),
            ),
            connection_capacity: self.concurrency_limit().max(1),
            hls_demand_expansion_allowed: self.concurrency.demand_expansion_allowed(),
            connection_ceiling: cycle.limits.global(),
            per_authority_request_limit: cycle.limits.per_authority(),
            packet_loss_bps: self.ctx.network.profile().packet_loss_bps,
            resource_feedback: Some(self.resources.feedback(environment)),
            capacity_revision: cycle.capacity.revision().value(),
            observed_at_ms: cycle.observed_at_ms,
            demanded: &cycle.demanded,
        };
        planned_work_with_planner(
            &self.state,
            &inputs,
            &mut self.warp_planner,
            self.qoe.watch_model(),
        )
    }

    fn resource_environment(&self, cycle: &PlanningCycle) -> ResourceEnvironment {
        let used = cycle
            .capacity
            .used_bytes()
            .saturating_add(cycle.segmented_storage_used_bytes);
        let limit = cycle
            .capacity
            .limit_bytes()
            .saturating_add(cycle.segmented_storage_capacity_bytes);
        let target = ResourceObservation::new(
            self.resource_network_target(),
            limit.saturating_mul(9) / 10,
            self.resource_cpu_target(),
            cycle.limits.global().max(1) as u64,
        );
        ResourceEnvironment::new(used, target)
    }

    fn resource_network_target(&self) -> u64 {
        let learned = self
            .keeper
            .stats()
            .overall_throughput()
            .map_or(OPTIMISTIC_THROUGHPUT_BPS as u64, |value| {
                value.bytes_per_second().min(u64::MAX as f64) as u64
            });
        let configured = self
            .ctx
            .network
            .profile()
            .bandwidth_kbps
            .saturating_mul(125);
        match configured {
            0 => learned.max(1),
            value => learned.min(value).max(1),
        }
    }

    fn resource_cpu_target(&self) -> u64 {
        self.state.transform_profile().map_or(0, |profile| {
            let hard = profile.limits().cpu_ms().min(500);
            ShadowPriceController::cpu_operating_target_ms(hard)
        })
    }

    pub(super) fn finish_reconcile(&mut self) {
        self.keeper.schedule_save(&self.ctx.events);
        self.reliability.observe(
            self.state.catalog().reliability_revision(),
            &self.ctx.events,
        );
    }
}
