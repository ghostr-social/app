//! Executes exact storage ranges selected by the adaptive policy.

use crate::manager::DeliveryWorker;
use ghostr_engine::adaptive::Eviction;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::ContentRevision;
use std::collections::{BTreeMap, HashMap};
use std::ops::Range;

impl DeliveryWorker {
    pub(super) async fn apply_policy_evictions(
        &mut self,
        evictions: &[Eviction],
        revisions: &HashMap<PostId, ContentRevision>,
    ) -> bool {
        for (post, ranges) in grouped(evictions) {
            let post = PostId::new(post);
            let Some(revision) = revisions.get(&post) else {
                return false;
            };
            if !self.apply_policy_eviction(post, ranges, *revision).await {
                return false;
            }
        }
        true
    }

    async fn apply_policy_eviction(
        &mut self,
        post: PostId,
        ranges: Vec<Range<u64>>,
        revision: ContentRevision,
    ) -> bool {
        let expected: u64 = ranges.iter().map(|range| range.end - range.start).sum();
        let result = self
            .ctx
            .store
            .evict_ranges_if_current(post.as_str(), &ranges, revision)
            .await;
        match result {
            Ok(outcome) if outcome.freed_bytes() == expected => {
                self.state.record_policy_evictions(post, outcome.ranges());
                true
            }
            Ok(_) => false,
            Err(error) => {
                log::warn!("Adaptive eviction failed for {}: {error:#}", post.as_str());
                false
            }
        }
    }
}

fn grouped(evictions: &[Eviction]) -> BTreeMap<String, Vec<Range<u64>>> {
    let mut grouped = BTreeMap::new();
    for eviction in evictions {
        grouped
            .entry(eviction.post.as_str().to_owned())
            .or_insert_with(Vec::new)
            .push(eviction.range.start..eviction.range.end);
    }
    grouped
}
