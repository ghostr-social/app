//! The manager's replanning pass, run after every event: read the
//! store, launch due probes, plan with the pure engine, then bring the
//! in-flight transfers in line with the freshly ordered plan.

use crate::manager::plan::{planned_work_with_planner, PlanInputs, PlannedWork};
use crate::manager::time::unix_time_ms;
use crate::manager::DeliveryWorker;
use ghostr_engine::adaptive::StorageSnapshot;
use ghostr_engine::{ByteRange, PostId};
use ghostr_partial_store::partial_range_store::ContentRevision;
use ghostr_partial_store::partial_range_store::StoredMediaSnapshot;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
struct PlanningStoreState {
    present: HashMap<PostId, Vec<ByteRange>>,
    finalized: HashSet<PostId>,
    totals: HashMap<PostId, u64>,
    continuation_sources: HashMap<PostId, String>,
    revisions: HashMap<PostId, ContentRevision>,
    snapshots: HashMap<PostId, StoredMediaSnapshot>,
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
        let segmented_limit = self.connection_ceiling();
        self.segmented.reconcile(
            self.ctx.client.clone(),
            self.ctx.events.clone(),
            segmented_limit,
            self.downloads.len(),
        );
        let capacity = self.ctx.store.capacity_snapshot().await;
        let window = self.collection_window(&capacity);
        let timeline_window = self.state.timeline_window_posts();
        let timeline_posts: HashSet<_> = timeline_window.iter().cloned().collect();
        let stored = self.collect_stored(&window, &timeline_posts).await;
        self.state.prune_player_preparations(&stored.revisions);
        let independent_sources = self.independent_objects.current(&stored.revisions);
        self.reconcile_timelines(&timeline_window, &stored.snapshots);
        self.reconcile_probe_bodies();
        let in_flight = self.downloads.actions();
        let active_head_probes = self.probes.active_identities();
        let demanded = self.resolve_gateway_demands(&stored.present);
        let connection_ceiling = self.connection_ceiling();
        let inputs = PlanInputs {
            stats: self.keeper.stats(),
            retry: &self.retry,
            present: &stored.present,
            finalized: &stored.finalized,
            stored_totals: &stored.totals,
            continuation_sources: &stored.continuation_sources,
            revisions: &stored.revisions,
            independent_sources: &independent_sources,
            completed_head_probes: self.probes.completed_posts(),
            in_flight: &in_flight,
            active_head_probes: &active_head_probes,
            storage: StorageSnapshot::new(capacity.limit_bytes(), capacity.used_bytes()),
            connection_capacity: self
                .concurrency_limit()
                .min(self.progressive_capacity())
                .max(1),
            connection_ceiling,
            packet_loss_bps: self.ctx.network.profile().packet_loss_bps,
            observed_at_ms,
            demanded: &demanded,
        };
        let planned = planned_work_with_planner(&mut self.state, inputs, &mut self.warp_planner);
        self.observe_plan(&planned, observed_at_ms);
        self.execute_planned_work(observed_at_ms, planned, &stored.revisions)
            .await;
        self.keeper.schedule_save(&self.ctx.events);
        self.reliability.observe(
            self.state.catalog().reliability_revision(),
            &self.ctx.events,
        );
    }

    async fn execute_planned_work(
        &mut self,
        observed_at_ms: u64,
        planned: PlannedWork,
        revisions: &HashMap<PostId, ContentRevision>,
    ) {
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
        self.reconcile_transfers(planned).await;
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
                if snapshot.binding() == Some(&binding) {
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
