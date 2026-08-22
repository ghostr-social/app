use super::progress::Pending;
use super::{targets, Active, SegmentedDelivery, Target, MAX_HLS_READY_WINDOW};
use crate::delivery_events::{DeliveryFocus, FocusItem};
use crate::segmented::cache::PreservedFocus;
use crate::segmented::SegmentedPhase;
use ghostr_engine::{DeliveryKind, PostId};
use std::collections::HashSet;

impl SegmentedDelivery {
    pub fn apply_focus(&mut self, focus: &DeliveryFocus) -> bool {
        let current = focus.current_index.min(focus.items.len().saturating_sub(1));
        let tracked = hls_items(&focus.items);
        let targets = targets(&focus.items, current, MAX_HLS_READY_WINDOW + 1);
        let delivery = focus.items.get(current).map(|item| item.meta.delivery);
        if self.equivalent(&tracked, &targets, delivery) {
            return false;
        }
        let generation = self.generation(focus);
        let preserved = self.reconcile_work(&targets);
        let protected = targets.iter().map(|target| target.post.clone()).collect();
        self.cache
            .reconcile_focus_window(generation, tracked.clone(), &protected, &preserved);
        self.tracked = tracked;
        self.targets = targets;
        self.current_delivery = delivery;
        self.seed_pending(generation);
        true
    }

    fn reconcile_work(&mut self, targets: &[Target]) -> PreservedFocus {
        let mut preserved = PreservedFocus::new();
        for (post, active) in &mut self.active {
            match retained_index(targets, post, &active.pending.root_source) {
                Some(index) if !active.cancelling => {
                    active.pending.source_index = index;
                    preserve(&mut preserved, post, &active.pending);
                }
                _ => cancel(active),
            }
        }
        self.pending.retain(|post, pending| {
            let Some(index) = retained_index(targets, post, &pending.root_source) else {
                return false;
            };
            pending.source_index = index;
            preserve(&mut preserved, post, pending);
            true
        });
        preserved
    }

    fn seed_pending(&mut self, generation: u64) {
        for target in self.targets.clone() {
            if self.cache.snapshot(target.post.as_str()).phase == SegmentedPhase::Ready
                || self.pending.contains_key(&target.post)
                || self
                    .active
                    .get(&target.post)
                    .is_some_and(|active| active_matches_target(&self.cache, active, &target))
            {
                continue;
            }
            self.seed_target(&target, generation);
        }
    }

    pub(crate) fn reseed_invalidated(&mut self) {
        for (post, generation) in self.cache.take_invalidated() {
            if self.cache.focus_generation(&post) != Some(generation) {
                continue;
            }
            let Some(target) = self
                .targets
                .iter()
                .find(|target| target.post == post)
                .cloned()
            else {
                continue;
            };
            self.pending.remove(&post);
            if let Some(active) = self.active.get_mut(&post) {
                cancel(active);
            }
            self.seed_target(&target, generation);
        }
    }

    fn seed_target(&mut self, target: &Target, generation: u64) {
        let Some(source) = target.sources.first() else {
            self.cache.mark_stage_failed(
                &target.post,
                generation,
                "HLS item has no source".to_owned(),
            );
            return;
        };
        let attempt = self.allocate_attempt();
        self.pending.insert(
            target.post.clone(),
            Pending::root(generation, attempt, 0, source.clone()),
        );
    }

    fn equivalent(
        &self,
        tracked: &[(PostId, Vec<String>)],
        targets: &[Target],
        delivery: Option<DeliveryKind>,
    ) -> bool {
        self.current_delivery == delivery && self.tracked == tracked && self.targets == targets
    }

    fn generation(&mut self, focus: &DeliveryFocus) -> u64 {
        self.next_generation = focus
            .generation
            .value()
            .unwrap_or_else(|| self.next_generation.saturating_add(1));
        self.next_generation
    }
}

fn active_matches_target(
    cache: &crate::segmented::SegmentedCache,
    active: &Active,
    target: &Target,
) -> bool {
    !active.cancelling
        && cache.focus_generation(&target.post) == Some(active.pending.generation)
        && crate::segmented::source_key::contains(&target.sources, &active.pending.root_source)
}

pub(super) fn hls_items(items: &[FocusItem]) -> Vec<(PostId, Vec<String>)> {
    let mut seen = HashSet::new();
    items
        .iter()
        .filter(|item| item.meta.delivery == DeliveryKind::Hls)
        .filter(|item| seen.insert(item.post.clone()))
        .map(|item| (item.post.clone(), item.meta.urls.clone()))
        .collect()
}

fn retained_index(targets: &[Target], post: &PostId, root: &str) -> Option<usize> {
    targets
        .iter()
        .find(|target| &target.post == post)?
        .sources
        .iter()
        .position(|source| {
            crate::segmented::source_key::canonical(source)
                == crate::segmented::source_key::canonical(root)
        })
}

fn preserve(preserved: &mut PreservedFocus, post: &PostId, pending: &Pending) {
    preserved.insert(
        post.clone(),
        (pending.generation, pending.root_source.clone()),
    );
}

fn cancel(active: &mut Active) {
    if active.cancelling {
        return;
    }
    if let Some(cancellation) = active.cancellation.take() {
        active.cancelling = cancellation.send(()).is_ok();
    }
}
