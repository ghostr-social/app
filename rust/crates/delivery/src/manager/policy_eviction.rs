//! Executes exact storage ranges selected by the adaptive policy.

use crate::manager::DeliveryWorker;
use ghostr_engine::adaptive::Eviction;
use std::collections::BTreeMap;
use std::ops::Range;

impl DeliveryWorker {
    pub(super) async fn apply_policy_evictions(&mut self, evictions: &[Eviction]) {
        for (post, ranges) in grouped(evictions) {
            match self.ctx.store.evict_ranges(&post, &ranges).await {
                Ok(0) => {}
                Ok(_) => self
                    .state
                    .record_policy_evictions(ghostr_engine::PostId::new(post), &ranges),
                Err(error) => {
                    log::warn!(
                        "Video store could not apply adaptive eviction for {post}: {error:#}"
                    )
                }
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
