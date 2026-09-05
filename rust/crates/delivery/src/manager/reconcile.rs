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
mod storage;

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
    pub(super) async fn reconcile(&mut self) {
        let observed_at_ms = unix_time_ms();
        self.apply_known_capability_fallbacks(observed_at_ms).await;
        self.select_playback_rendition(observed_at_ms).await;
        let request_limits = self.request_limits();
        self.reconcile_request_surfaces(request_limits);
        let cycle = self
            .prepare_planning_cycle(observed_at_ms, request_limits)
            .await;
        let planned = self.plan_cycle(&cycle);
        self.observe_planner_cpu(&planned);
        Box::pin(self.execute_planned_work(observed_at_ms, planned, &cycle.stored)).await;
        self.finish_reconcile();
    }

    async fn execute_planned_work(
        &mut self,
        observed_at_ms: u64,
        planned: PlannedWork,
        stored: &PlanningStoreState,
    ) {
        if self.reclaim_cold_storage(&planned).await {
            self.request_immediate_replan();
            return;
        }
        self.schedule_hedge_tail_wakes(&planned.hedge_tails);
        self.schedule_network_refill_wake(planned.network_refill_deadline_ms);
        if !self
            .apply_policy_evictions(&planned.evictions, &stored.revisions)
            .await
        {
            return;
        }
        let startups = self
            .prepare_applied_plan(observed_at_ms, &planned, stored)
            .await;
        let receipt = self.observe_plan(&planned, observed_at_ms);
        let decision_sequence = receipt.as_ref().map(|receipt| receipt.sequence());
        let decision = receipt.and_then(|receipt| receipt.into_token());
        self.publish_applied_plan(observed_at_ms, &planned, startups, decision_sequence);
        self.reconcile_transfers(planned, decision, observed_at_ms)
            .await;
    }

    async fn prepare_applied_plan(
        &mut self,
        observed_at_ms: u64,
        planned: &PlannedWork,
        stored: &PlanningStoreState,
    ) -> Vec<crate::startup_certificate::StartupCertificate> {
        self.additional_request_slot_demand = planned
            .warp
            .as_ref()
            .map(|decision| decision.additional_request_slot_demanded);
        self.state
            .update_ready_target(planned.plan.ready_reserve.target);
        self.state
            .observe_discovery_demand(planned.discovery_demand);
        self.refresh_cache_registry(observed_at_ms).await;
        self.startup_certificates(&planned.plan, &stored.snapshots)
    }

    fn publish_applied_plan(
        &self,
        observed_at_ms: u64,
        planned: &PlannedWork,
        startups: Vec<crate::startup_certificate::StartupCertificate>,
        decision_sequence: Option<u64>,
    ) {
        let publication = crate::delivery_events::PlanPublicationContext::new(
            observed_at_ms,
            self.state.current_post(),
        )
        .with_player_preparations(planned.player_preparations.clone())
        .with_focus(
            self.state.focus_generation(),
            self.state.focus_covers_from(),
        )
        .with_network(
            self.state.network_status(),
            self.state.network_profile_generation(),
        )
        .with_decision_sequence(decision_sequence);
        self.commands.publish_causal_plan_with_startups(
            publication,
            planned.plan.clone(),
            startups,
        );
    }

    fn reconcile_probe_bodies(&mut self) {
        let active = self.downloads.body_identities();
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
        let mut ordered: Vec<_> = demanded.iter().collect();
        ordered.sort_by(|left, right| left.0.cmp(right.0));
        for (post, range) in ordered {
            self.expedite_demand(post, range.start);
        }
        demanded
    }
}
