//! The manager's replanning pass, run after every event: read the
//! store, launch due probes, plan with the pure engine, then bring the
//! in-flight transfers in line with the freshly ordered plan.

use crate::manager::plan::PlannedWork;
use crate::manager::time::unix_time_ms;
use crate::manager::DeliveryWorker;
use ghostr_engine::{ByteRange, PostId};
use ghostr_partial_store::partial_range_store::ContentRevision;
use ghostr_partial_store::partial_range_store::StoredMediaSnapshot;
use std::collections::{HashMap, HashSet};

mod cycle;

#[derive(Default)]
struct PlanningStoreState {
    present: HashMap<PostId, Vec<ByteRange>>,
    finalized: HashSet<PostId>,
    totals: HashMap<PostId, u64>,
    continuation_sources: HashMap<PostId, String>,
    revisions: HashMap<PostId, ContentRevision>,
    snapshots: HashMap<PostId, StoredMediaSnapshot>,
    transformed: HashMap<PostId, ghostr_engine::representation::RepresentationBinding>,
}

struct PlannedExecution {
    planned: PlannedWork,
    decision: Option<crate::delivery_events::DecisionToken>,
}

impl PlanningStoreState {
    fn insert(&mut self, post: PostId, snapshot: StoredMediaSnapshot, retain_snapshot: bool) {
        let ranges = snapshot
            .planning_ranges()
            .iter()
            .map(|span| ByteRange::new(span.start, span.end))
            .collect();
        self.present.insert(post.clone(), ranges);
        if snapshot.is_finalized() {
            self.finalized.insert(post.clone());
        }
        self.revisions.insert(post.clone(), snapshot.revision());
        if let Some(total) = snapshot.total_len() {
            self.totals.insert(post.clone(), total);
        }
        if let Some(source) = snapshot.continuation_source() {
            self.continuation_sources
                .insert(post.clone(), source.to_owned());
        }
        if retain_snapshot {
            self.snapshots.insert(post, snapshot);
        }
    }
}

impl DeliveryWorker {
    pub(crate) async fn reconcile(&mut self) {
        let observed_at_ms = unix_time_ms();
        self.select_playback_rendition(observed_at_ms).await;
        let request_limits = self.request_limits();
        self.reconcile_request_surfaces(request_limits);
        let cycle = self
            .prepare_planning_cycle(observed_at_ms, request_limits)
            .await;
        let planned = self.plan_cycle(&cycle);
        let decision = self.observe_plan(&planned, observed_at_ms);
        let execution = PlannedExecution { planned, decision };
        self.execute_planned_work(observed_at_ms, execution, &cycle.stored.revisions)
            .await;
        self.finish_reconcile();
    }

    async fn execute_planned_work(
        &mut self,
        observed_at_ms: u64,
        execution: PlannedExecution,
        revisions: &HashMap<PostId, ContentRevision>,
    ) {
        let PlannedExecution { planned, decision } = execution;
        self.schedule_hedge_tail_wakes(&planned.hedge_tails, observed_at_ms);
        self.schedule_network_refill_wake(planned.network_refill_deadline_ms);
        self.additional_request_slot_demand = planned
            .warp
            .as_ref()
            .map(|decision| decision.additional_request_slot_demanded);
        if !self
            .apply_policy_evictions(&planned.evictions, revisions)
            .await
        {
            return;
        }
        self.state
            .update_ready_target(planned.plan.ready_reserve.target);
        self.state
            .observe_discovery_demand(planned.discovery_demand);
        self.refresh_cache_registry().await;
        let startups = self.startup_certificates(&planned.plan).await;
        self.commands.publish_focused_plan_with_startups(
            observed_at_ms,
            self.state.current_post(),
            planned.plan.clone(),
            startups,
        );
        self.reconcile_transfers(planned, decision).await;
    }

    /// The planning slice of the window, widened to the full roster
    /// only under storage pressure so eviction can weigh every stored
    /// post, not just the current neighbourhood.
    fn collection_window(
        &self,
        capacity: &ghostr_partial_store::partial_range_store::capacity::CapacitySnapshot,
    ) -> Vec<PostId> {
        if capacity.used_bytes() >= capacity.limit_bytes().saturating_mul(9) / 10 {
            return self.state.window_posts();
        }
        self.state.planning_window_posts()
    }

    async fn collect_stored(
        &self,
        window: &[PostId],
        timeline_posts: &HashSet<PostId>,
    ) -> PlanningStoreState {
        let mut stored = PlanningStoreState::default();
        for post in window {
            let Some(binding) = self.state.catalog().binding(post) else {
                continue;
            };
            if let Ok(snapshot) = self.ctx.store.media_snapshot(post.as_str()).await {
                if snapshot
                    .binding()
                    .is_some_and(|stored| stored == &binding || stored.derives_from(&binding))
                {
                    if let Some(transformed) = snapshot
                        .binding()
                        .filter(|stored| stored.derives_from(&binding))
                    {
                        stored.transformed.insert(post.clone(), transformed.clone());
                    }
                    stored.insert(post.clone(), snapshot, timeline_posts.contains(post));
                }
            }
        }
        stored
    }

    fn reconcile_probe_bodies(&mut self) {
        let active = self.downloads.body_posts();
        self.probes.reconcile_bodies(&active);
    }

    fn resolve_gateway_demands(
        &mut self,
        present: &HashMap<PostId, Vec<ByteRange>>,
    ) -> HashMap<PostId, ByteRange> {
        let foreground = self.state.demand_posts();
        let demanded = self
            .demand_leases
            .reconcile(&foreground, self.state.catalog(), present);
        for (post, range) in &demanded {
            self.expedite_demand(post, range.start);
        }
        demanded
    }
}
