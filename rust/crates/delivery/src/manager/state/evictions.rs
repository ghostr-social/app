use super::DeliveryState;
use ghostr_engine::{ByteRange, PostId};
use std::ops::Range;

impl DeliveryState {
    pub(crate) fn record_policy_evictions(&mut self, post: PostId, ranges: &[Range<u64>]) {
        let tracked = self.recent_evictions.entry(post).or_default();
        tracked.extend(
            ranges
                .iter()
                .filter(|range| range.start < range.end)
                .map(|range| ByteRange::new(range.start, range.end)),
        );
        *tracked = ghostr_engine::media_timeline::normalize(std::mem::take(tracked));
    }

    pub(crate) fn recently_evicted(&self, post: &PostId) -> Vec<ByteRange> {
        self.recent_evictions.get(post).cloned().unwrap_or_default()
    }

    pub(super) fn forget_evictions(&mut self, post: &PostId) {
        self.recent_evictions.remove(post);
    }

    pub(super) fn retain_evictions(&mut self, retained: &std::collections::HashSet<PostId>) {
        self.recent_evictions
            .retain(|post, _| retained.contains(post));
    }
}
