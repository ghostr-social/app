//! The manager's replanning pass, run after every event: read the
//! store, launch due probes, plan with the pure engine, then bring the
//! in-flight transfers in line with the freshly ordered plan.

use crate::manager::plan::{planned_work, PlanInputs};
use crate::manager::time::unix_time_ms;
use crate::manager::transfers::spawn_probe;
use crate::manager::DeliveryWorker;
use ghostr_engine::adaptive::StorageSnapshot;
use ghostr_engine::{ByteRange, PostId};
use std::collections::HashMap;

impl DeliveryWorker {
    pub(crate) async fn reconcile(&mut self) {
        let observed_at_ms = unix_time_ms();
        self.select_playback_rendition(observed_at_ms).await;
        let capacity = self.ctx.store.capacity_snapshot().await;
        let window = self.collection_window(&capacity);
        let probe_posts = self.state.probe_posts();
        let present = self.collect_present(&window).await;
        self.hydrate_timelines(&window, &present).await;
        self.ensure_total_lens(&window).await;
        self.reconcile_probes(&probe_posts);
        let in_flight = self.downloads.ranges();
        let demanded = self.resolve_gateway_demands(&present);
        let connection_ceiling = self.connection_ceiling();
        let inputs = PlanInputs {
            stats: self.keeper.stats(),
            retry: &self.retry,
            present: &present,
            in_flight: &in_flight,
            storage: StorageSnapshot::new(capacity.limit_bytes(), capacity.used_bytes()),
            connection_capacity: self.concurrency_limit().min(connection_ceiling),
            connection_ceiling,
            packet_loss_bps: self.ctx.network.profile().packet_loss_bps,
            observed_at_ms,
            demanded: &demanded,
        };
        let planned = planned_work(&mut self.state, inputs);
        self.commands
            .publish_plan(observed_at_ms, planned.plan.clone());
        self.apply_policy_evictions(&planned.evictions).await;
        self.state
            .observe_discovery_demand(planned.discovery_demand);
        self.reconcile_transfers(planned);
        self.refresh_cache_registry().await;
        self.keeper.schedule_save(&self.ctx.events);
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

    async fn collect_present(&self, window: &[PostId]) -> HashMap<PostId, Vec<ByteRange>> {
        let mut present = HashMap::new();
        for post in window {
            if self.state.catalog().lookup(post).is_none() {
                continue;
            }
            let spans = self
                .ctx
                .store
                .present_ranges(post.as_str())
                .await
                .unwrap_or_default();
            let ranges = spans
                .into_iter()
                .map(|span| ByteRange::new(span.start, span.end))
                .collect();
            present.insert(post.clone(), ranges);
        }
        present
    }

    /// The store must know the total before the gateway's first serve;
    /// declare it as soon as the catalog knows (imeta or probe).
    async fn ensure_total_lens(&mut self, window: &[PostId]) {
        for post in window {
            let known = self
                .state
                .catalog()
                .lookup(post)
                .and_then(|e| e.total_bytes());
            let Some(total) = known else { continue };
            self.set_store_total(post, total).await;
        }
    }

    fn launch_probes(&mut self, window: &[PostId]) {
        for (post, url) in self.probes.claim(self.state.catalog(), window, &self.retry) {
            spawn_probe(self.ctx.clone(), post, url);
        }
    }

    fn reconcile_probes(&mut self, window: &[PostId]) {
        let active = self.downloads.body_posts();
        self.probes.reconcile_bodies(&active);
        self.launch_probes(window);
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
