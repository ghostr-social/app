use super::fetch::{FetchFailure, FetchedObject};
use super::{SegmentedCache, SegmentedPhase};
use crate::delivery_events::{DeliveryFocus, FocusItem};
use crate::manager::traffic::TrafficPublisher;
use crate::manager::transfers::InternalEvent;
use ghostr_engine::adaptive::HlsBootstrapStage;
use ghostr_engine::origin_model::OriginObservation;
use ghostr_engine::{ActionId, DeliveryKind, PostId};
use ghostr_net::media_request_executor::MediaRequestExecutor;
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc::UnboundedSender;

#[cfg(test)]
#[path = "scheduler/test_registry.rs"]
mod test_registry;

mod completion;
mod launch;
mod progress;
mod resources;
mod snapshot;
mod target;

use progress::Pending;
pub(crate) use resources::SegmentedResourceCommitment;
use target::{targets, Target};

const MAX_HLS_READY_WINDOW: usize = 5;

pub(crate) struct SegmentedDelivery {
    cache: SegmentedCache,
    tracked: Vec<(PostId, Vec<String>)>,
    targets: Vec<Target>,
    pending: HashMap<PostId, Pending>,
    active: HashMap<PostId, Active>,
    next_generation: u64,
    current_delivery: Option<DeliveryKind>,
    startup_eta_ms: u64,
}

struct Active {
    action: ActionId,
    pending: Pending,
    committed_until_ms: u64,
    _task: tokio::task::JoinHandle<()>,
    cancellation: Option<tokio::sync::oneshot::Sender<()>>,
    cancelling: bool,
}

pub(crate) struct SegmentedLaunch {
    pub post: PostId,
    pub stage: HlsBootstrapStage,
    pub source: String,
    pub maximum_bytes: u64,
    pub committed_until_ms: u64,
    pub action: ActionId,
    pub requests: MediaRequestExecutor,
    pub events: UnboundedSender<InternalEvent>,
    pub network_status: crate::delivery_events::DeliveryNetworkStatusReader,
    pub traffic: TrafficPublisher,
    pub resources: SegmentedResourceCommitment,
}

pub(crate) struct SegmentedDone {
    action: ActionId,
    post: PostId,
    generation: u64,
    outcome: Result<FetchedObject, FetchFailure>,
    observed_at_ms: u64,
    resources: SegmentedResourceCommitment,
}

pub(crate) struct SegmentedFinish {
    pub action: ActionId,
    pub outcome: ghostr_engine::adaptive::DecisionOutcome,
    pub observation: Option<OriginObservation>,
    pub actual_resources: Option<ghostr_engine::adaptive::ResourceCost>,
    pub resources: SegmentedResourceCommitment,
}

impl SegmentedDelivery {
    pub fn new(cache: SegmentedCache) -> Self {
        Self {
            cache,
            tracked: Vec::new(),
            targets: Vec::new(),
            pending: HashMap::new(),
            active: HashMap::new(),
            next_generation: 0,
            current_delivery: None,
            startup_eta_ms: crate::qoe::QoeTracker::DEFAULT_STARTUP_ETA_MS,
        }
    }

    pub fn apply_focus(&mut self, focus: &DeliveryFocus) -> bool {
        let current = focus.current_index.min(focus.items.len().saturating_sub(1));
        let tracked = hls_items(&focus.items);
        let targets = targets(&focus.items, current, MAX_HLS_READY_WINDOW + 1);
        let current_delivery = focus.items.get(current).map(|item| item.meta.delivery);
        if self.equivalent(&tracked, &targets, current_delivery) {
            return false;
        }
        let generation = self.generation(focus);
        self.cancel_all();
        let protected = targets.iter().map(|target| target.post.clone()).collect();
        self.cache
            .replace_focus_window(generation, tracked.clone(), &protected);
        self.tracked = tracked;
        self.targets = targets;
        self.current_delivery = current_delivery;
        self.pending.clear();
        self.seed_pending(generation);
        true
    }

    pub fn active_len(&self) -> usize {
        self.active.len()
    }

    pub fn set_startup_eta_ms(&mut self, eta_ms: u64) {
        self.startup_eta_ms = eta_ms;
    }

    pub fn clear(&mut self) {
        self.cancel_all();
        self.tracked.clear();
        self.targets.clear();
        self.pending.clear();
        self.current_delivery = None;
        self.cache.clear();
    }

    fn seed_pending(&mut self, generation: u64) {
        for target in &self.targets {
            if self.cache.snapshot(target.post.as_str()).phase == SegmentedPhase::Ready {
                continue;
            }
            let Some(source) = target.sources.first() else {
                self.cache.mark_stage_failed(
                    &target.post,
                    generation,
                    "HLS item has no source".to_owned(),
                );
                continue;
            };
            self.pending.insert(
                target.post.clone(),
                Pending::root(generation, 0, source.clone()),
            );
        }
    }

    fn equivalent(
        &self,
        tracked: &[(PostId, Vec<String>)],
        targets: &[Target],
        current_delivery: Option<DeliveryKind>,
    ) -> bool {
        self.current_delivery == current_delivery
            && self.tracked == tracked
            && self.targets == targets
    }

    fn generation(&mut self, focus: &DeliveryFocus) -> u64 {
        self.next_generation = focus
            .generation
            .value()
            .unwrap_or_else(|| self.next_generation.saturating_add(1));
        self.next_generation
    }

    fn cancel_all(&mut self) {
        for active in self.active.values_mut() {
            if active.cancelling {
                continue;
            }
            if let Some(cancellation) = active.cancellation.take() {
                active.cancelling = cancellation.send(()).is_ok();
            }
        }
    }
}

fn hls_items(items: &[FocusItem]) -> Vec<(PostId, Vec<String>)> {
    let mut seen = HashSet::new();
    items
        .iter()
        .filter(|item| item.meta.delivery == DeliveryKind::Hls)
        .filter(|item| seen.insert(item.post.clone()))
        .map(|item| (item.post.clone(), item.meta.urls.clone()))
        .collect()
}
