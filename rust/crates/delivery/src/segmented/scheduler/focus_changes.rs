use super::{hls_items, SegmentedDelivery};
use crate::delivery_events::DeliveryFocus;
use ghostr_engine::PostId;
use std::collections::HashSet;

impl SegmentedDelivery {
    pub(crate) fn changed_hls_sources(&self, focus: &DeliveryFocus) -> Vec<PostId> {
        let next = hls_items(&focus.items);
        let mut seen = HashSet::new();
        self.tracked
            .iter()
            .chain(next.iter())
            .map(|(post, _)| post)
            .filter(|post| seen.insert((*post).clone()))
            .filter(|post| changed(roots(&self.tracked, post), roots(&next, post)))
            .cloned()
            .collect()
    }

    pub(crate) fn tracked_roots(&self, post: &PostId) -> Vec<String> {
        roots(&self.tracked, post).cloned().unwrap_or_default()
    }

    pub(crate) fn hls_cooldown_resets(&self, focus: &DeliveryFocus) -> Vec<PostId> {
        let next = hls_items(&focus.items);
        self.tracked
            .iter()
            .filter(|(post, previous)| {
                should_reset(
                    previous,
                    roots(&next, post).map(Vec::as_slice).unwrap_or_default(),
                    self.selected_root(post).as_deref(),
                )
            })
            .map(|(post, _)| post.clone())
            .collect()
    }

    fn selected_root(&self, post: &PostId) -> Option<String> {
        self.active
            .get(post)
            .filter(|active| !active.cancelling)
            .map(|active| active.pending.root_source.clone())
            .or_else(|| {
                self.pending
                    .get(post)
                    .map(|pending| pending.root_source.clone())
            })
            .or_else(|| self.cache.root_source(post))
    }
}

fn changed(previous: Option<&Vec<String>>, next: Option<&Vec<String>>) -> bool {
    match (previous, next) {
        (Some(previous), Some(next)) => !crate::segmented::source_key::same_members(previous, next),
        (None, None) => false,
        (Some(_), None) | (None, Some(_)) => true,
    }
}

fn should_reset(previous: &[String], next: &[String], selected: Option<&str>) -> bool {
    let selected_removed =
        selected.is_some_and(|root| !crate::segmented::source_key::contains(next, root));
    let disjoint = !previous.is_empty()
        && !next.is_empty()
        && !previous
            .iter()
            .any(|root| crate::segmented::source_key::contains(next, root));
    let new_root = next
        .iter()
        .any(|root| !crate::segmented::source_key::contains(previous, root));
    selected_removed || next.is_empty() || disjoint || new_root
}

fn roots<'a>(items: &'a [(PostId, Vec<String>)], post: &PostId) -> Option<&'a Vec<String>> {
    items
        .iter()
        .find(|(candidate, _)| candidate == post)
        .map(|(_, roots)| roots)
}
