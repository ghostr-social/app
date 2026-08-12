//! The manager's replanning pass, run after every event: read the
//! store, launch due probes, plan with the pure engine, then bring the
//! in-flight transfers in line with the freshly ordered plan.

use crate::manager::concurrency::planned_capacity;
use crate::manager::plan::{planned_work, PlanInputs, PlannedTransfer, PlannedWork};
use crate::manager::transfers::spawn_probe;
use crate::manager::DeliveryWorker;
use crate::mutable_priority_queue::ForegroundSlots;
use crate::playback_demand::DemandSignal;
use ghostr_engine::tiers::DemandSignals;
use ghostr_engine::{ByteRange, PostId};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

impl DeliveryWorker {
    pub(crate) async fn reconcile(&mut self) {
        let observed_at_ms = unix_time_ms();
        self.select_playback_rendition(observed_at_ms).await;
        let window = self.state.window_posts();
        let candidates = self.state.candidate_posts();
        let probe_posts = self.state.probe_posts();
        let present = self.collect_present(&window).await;
        self.hydrate_timelines(&window, &present).await;
        self.ensure_total_lens(&candidates).await;
        self.launch_probes(&probe_posts);
        let demand = self.demand_signals(&present);
        let demanded = self.pending_demand.clone();
        let inputs = PlanInputs {
            stats: self.keeper.stats(),
            retry: &self.retry,
            present: &present,
            demand,
            observed_at_ms,
            demanded,
        };
        let planned = planned_work(&mut self.state, inputs);
        self.reconcile_transfers(planned);
        self.refresh_cache_registry().await;
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

    /// Live gateway demand counts while it concerns the playing post
    /// and the demanded bytes are still missing; stale demand drops.
    fn demand_signals(&mut self, present: &HashMap<PostId, Vec<ByteRange>>) -> DemandSignals {
        let signals = resolve_demand(
            &mut self.pending_demand,
            self.state.focus().current(),
            present,
        );
        if signals.gateway_demand {
            let (post, offset) = self
                .pending_demand
                .as_ref()
                .map(|signal| (signal.post.clone(), signal.range.start))
                .expect("active demand");
            self.expedite_demand(&post, offset);
        }
        signals
    }

    /// Reconciles active IO against the fresh locality and priority plan,
    /// then grants ordered work up to the concurrency limit.
    fn reconcile_transfers(&mut self, planned: PlannedWork) {
        let emergency = planned.emergency;
        let eviction = planned.eviction;
        let capacity = planned_capacity(
            self.concurrency_limit(),
            self.state.concurrency(),
            &planned.transfers,
        );
        let priority: Vec<_> = planned
            .transfers
            .iter()
            .map(|transfer| transfer.request.chunk.clone())
            .collect();
        self.downloads.reconcile_with_commitments(
            &planned.transfers,
            capacity.total,
            eviction,
            &planned.protected_identities,
        );
        self.queue.replace(planned.transfers);
        if let Some(current) = self.state.focus().current() {
            self.downloads
                .preempt_for_current(current, &priority, capacity.total, eviction);
        }
        while self.downloads.len() < capacity.total {
            let active_hosts = self.downloads.active_hosts();
            let foreground =
                ForegroundSlots::new(self.downloads.foreground_len(), capacity.foreground_goal);
            let Some(transfer) = self.queue.pop_for_hosts(&active_hosts, foreground) else {
                return;
            };
            self.grant(transfer);
        }
        if !emergency {
            self.grant_origin_exploration();
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
}

fn unix_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u128::from(u64::MAX)) as u64
}

pub(crate) fn resolve_demand(
    pending: &mut Option<DemandSignal>,
    playing: Option<&PostId>,
    present: &HashMap<PostId, Vec<ByteRange>>,
) -> DemandSignals {
    let Some(signal) = pending.as_ref() else {
        return DemandSignals::default();
    };
    if playing != Some(&signal.post) || covered(signal.range, present.get(&signal.post)) {
        *pending = None;
        return DemandSignals::default();
    }
    DemandSignals {
        gateway_demand: true,
        ..DemandSignals::default()
    }
}

/// Whether the store's (coalesced) ranges fully cover `range`.
fn covered(range: ByteRange, have: Option<&Vec<ByteRange>>) -> bool {
    let Some(have) = have else { return false };
    have.iter()
        .any(|span| span.start <= range.start && span.end >= range.end)
}
