use super::{prepare_hls, SegmentedCache, SegmentedPhase};
use crate::delivery_events::DeliveryFocus;
use crate::manager::transfers::{InternalEvent, SegmentedDone};
use ghostr_engine::{DeliveryKind, PostId};
use ghostr_net::outbound_media_client::MediaHttpRequests;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

#[cfg(test)]
mod tests;

const MAX_HLS_READY_WINDOW: usize = 5;

pub(crate) struct SegmentedDelivery {
    cache: SegmentedCache,
    tracked: Vec<(PostId, Vec<String>)>,
    targets: Vec<Target>,
    active: HashMap<PostId, Active>,
    next_generation: u64,
    current_delivery: Option<DeliveryKind>,
    startup_eta_ms: u64,
}

#[derive(Clone, Eq, PartialEq)]
struct Target {
    post: PostId,
    sources: Vec<String>,
}

struct Active {
    generation: u64,
    task: tokio::task::JoinHandle<()>,
}

impl SegmentedDelivery {
    pub fn new(cache: SegmentedCache) -> Self {
        Self {
            cache,
            tracked: Vec::new(),
            targets: Vec::new(),
            active: HashMap::new(),
            next_generation: 0,
            current_delivery: None,
            startup_eta_ms: crate::qoe::QoeTracker::DEFAULT_STARTUP_ETA_MS,
        }
    }

    pub fn apply_focus(&mut self, focus: &DeliveryFocus) {
        let current = focus.current_index.min(focus.items.len().saturating_sub(1));
        let tracked = hls_items(&focus.items);
        let targets: Vec<Target> = focus.items[current..]
            .iter()
            .take(MAX_HLS_READY_WINDOW + 1)
            .filter(|item| item.meta.delivery == DeliveryKind::Hls)
            .map(|item| Target {
                post: item.post.clone(),
                sources: item.meta.urls.clone(),
            })
            .collect();
        let current_delivery = focus.items.get(current).map(|item| item.meta.delivery);
        if self.equivalent(&tracked, &targets, current_delivery) {
            return;
        }
        let generation = self.generation(focus);
        self.abort_all();
        self.cache.replace_focus(generation, tracked.clone());
        self.tracked = tracked;
        self.targets = targets;
        self.current_delivery = current_delivery;
    }

    pub fn reconcile(
        &mut self,
        client: Arc<dyn MediaHttpRequests>,
        events: UnboundedSender<InternalEvent>,
        connection_limit: usize,
        progressive_active: usize,
    ) {
        let reserve = usize::from(
            self.current_delivery == Some(DeliveryKind::Progressive) && progressive_active == 0,
        );
        let capacity = connection_limit
            .saturating_sub(progressive_active)
            .saturating_sub(reserve);
        let generation = self.next_generation;
        for target in self.targets.clone() {
            if self.active.len() >= capacity {
                break;
            }
            if self.active.contains_key(&target.post) || !self.should_start(&target.post) {
                continue;
            }
            self.start(target, generation, client.clone(), events.clone());
        }
    }

    pub fn finish(&mut self, done: SegmentedDone) {
        let current = self
            .active
            .get(&done.post)
            .is_some_and(|active| active.generation == done.generation);
        if current {
            self.active.remove(&done.post);
        }
    }

    pub fn active_len(&self) -> usize {
        self.active.len()
    }

    pub fn set_startup_eta_ms(&mut self, eta_ms: u64) {
        self.startup_eta_ms = eta_ms;
    }

    pub fn clear(&mut self) {
        self.abort_all();
        self.tracked.clear();
        self.targets.clear();
        self.current_delivery = None;
        self.cache.clear();
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

    fn should_start(&self, post: &PostId) -> bool {
        matches!(
            self.cache.snapshot(post.as_str()).phase,
            SegmentedPhase::Queued
        )
    }

    fn start(
        &mut self,
        target: Target,
        generation: u64,
        client: Arc<dyn MediaHttpRequests>,
        events: UnboundedSender<InternalEvent>,
    ) {
        if !self
            .cache
            .mark_preparing(&target.post, generation, self.startup_eta_ms)
        {
            return;
        }
        let cache = self.cache.clone();
        let post = target.post.clone();
        let task_post = post.clone();
        let task = tokio::spawn(async move {
            let result = prepare_hls(client.as_ref(), &target.sources).await;
            cache.complete(&task_post, generation, result);
            let _ = events.send(InternalEvent::Segmented(SegmentedDone {
                post: task_post,
                generation,
            }));
        });
        self.active.insert(post, Active { generation, task });
    }

    fn abort_all(&mut self) {
        for (_, active) in self.active.drain() {
            active.task.abort();
        }
    }
}

fn hls_items(items: &[crate::delivery_events::FocusItem]) -> Vec<(PostId, Vec<String>)> {
    let mut seen = HashSet::new();
    items
        .iter()
        .filter(|item| item.meta.delivery == DeliveryKind::Hls)
        .filter(|item| seen.insert(item.post.clone()))
        .map(|item| (item.post.clone(), item.meta.urls.clone()))
        .collect()
}
