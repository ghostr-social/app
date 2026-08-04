//! The manager's replanning pass, run after every event: read the
//! store, launch due probes, plan with the pure engine, then bring the
//! in-flight transfers in line with the freshly ordered plan.

use crate::engine::scoring::ChunkRequest;
use crate::engine::tiers::DemandSignals;
use crate::engine::{ByteRange, ChunkId, PostId};
use crate::video::delivery_manager::DeliveryWorker;
use crate::video::delivery_plan::{planned_work, PlanInputs, PlannedWork};
use crate::video::delivery_transfers::{spawn_chunk, spawn_probe};
use std::collections::{HashMap, HashSet};

impl DeliveryWorker {
    pub(crate) async fn reconcile(&mut self) {
        let window = self.state.window_posts();
        let present = self.collect_present(&window).await;
        self.ensure_total_lens(&window).await;
        self.launch_probes(&window);
        let demand = self.demand_signals(&present);
        let inputs = PlanInputs {
            stats: self.keeper.stats(),
            retry: &self.retry,
            present: &present,
            demand,
        };
        let planned = planned_work(&mut self.state, inputs);
        self.reconcile_transfers(planned);
        self.keeper.schedule_save(&self.ctx.events);
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
            let known = self.state.catalog().lookup(post).and_then(|e| e.total_bytes());
            let Some(total) = known else { continue };
            self.set_store_total(post, total).await;
        }
    }

    fn launch_probes(&mut self, window: &[PostId]) {
        for (post, url) in self.probes.claim(self.state.catalog(), window, &self.retry) {
            spawn_probe(self.ctx.clone(), post, url);
        }
    }

    /// Live gateway demand counts while it concerns the playing post
    /// and the demanded bytes are still missing; stale demand drops.
    fn demand_signals(&mut self, present: &HashMap<PostId, Vec<ByteRange>>) -> DemandSignals {
        let Some(signal) = self.pending_demand.clone() else {
            return DemandSignals::default();
        };
        let playing = self.state.focus().current() == Some(&signal.post);
        if !playing || covered(signal.range, present.get(&signal.post)) {
            self.pending_demand = None;
            return DemandSignals::default();
        }
        DemandSignals {
            gateway_demand: true,
            ..DemandSignals::default()
        }
    }

    /// Grants the ordered plan up to the concurrency limit; whatever
    /// the plan no longer contains is cancelled first (scroll-past).
    fn reconcile_transfers(&mut self, planned: PlannedWork) {
        let wanted: HashSet<ChunkId> = planned
            .requests
            .iter()
            .map(|request| request.chunk.clone())
            .collect();
        self.inflight.cancel_absent(&wanted);
        for request in &planned.requests {
            if self.inflight.len() >= self.state.concurrency() {
                break;
            }
            self.grant(request, &planned.urls);
        }
    }

    fn grant(&mut self, request: &ChunkRequest, urls: &HashMap<PostId, String>) {
        let chunk = &request.chunk;
        if self.inflight.contains(chunk) || self.retry.is_cooling(&chunk.post) {
            return;
        }
        let Some(url) = urls.get(&chunk.post) else { return };
        let handle = spawn_chunk(self.ctx.clone(), chunk.clone(), url.clone());
        self.inflight.insert(chunk.clone(), handle);
    }
}

/// Whether the store's (coalesced) ranges fully cover `range`.
fn covered(range: ByteRange, have: Option<&Vec<ByteRange>>) -> bool {
    let Some(have) = have else { return false };
    have.iter()
        .any(|span| span.start <= range.start && span.end >= range.end)
}
