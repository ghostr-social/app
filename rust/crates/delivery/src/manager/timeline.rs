//! Reads only bounded metadata-bearing sparse spans and installs parsed
//! MP4/CMAF timing evidence into the representation-fenced catalog.

use crate::manager::DeliveryWorker;
use ghostr_engine::media_timeline::{parse_mp4_segments, MediaSegment, MediaTimeline};
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::{ByteRange, PostId};
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::collections::HashMap;

const MAX_METADATA_SPAN: u64 = 4 * 1024 * 1024;

impl DeliveryWorker {
    pub(super) async fn hydrate_timelines(
        &mut self,
        posts: &[PostId],
        present: &HashMap<PostId, Vec<ByteRange>>,
    ) {
        for post in posts {
            self.hydrate_timeline(post, present.get(post).map(Vec::as_slice).unwrap_or(&[]))
                .await;
        }
    }

    async fn hydrate_timeline(&mut self, post: &PostId, present: &[ByteRange]) {
        let Some((binding, total)) = timeline_target(&self.state, post) else {
            return;
        };
        let Some(timeline) = load_timeline(&self.ctx.store, post, total, present).await else {
            require_tail_if_started(&mut self.state, &binding, present);
            return;
        };
        install_timeline(&mut self.state, &binding, timeline);
    }
}

pub(crate) fn install_timeline(
    state: &mut crate::manager::state::DeliveryState,
    binding: &RepresentationBinding,
    timeline: MediaTimeline,
) -> bool {
    state.catalog_mut().learn_timeline_for(binding, timeline)
}

fn timeline_target(
    state: &crate::manager::state::DeliveryState,
    post: &PostId,
) -> Option<(RepresentationBinding, u64)> {
    let entry = state.catalog().lookup(post)?;
    if entry.timeline().is_some() {
        return None;
    }
    Some((entry.binding(), entry.total_bytes()?))
}

fn require_tail_if_started(
    state: &mut crate::manager::state::DeliveryState,
    binding: &RepresentationBinding,
    present: &[ByteRange],
) {
    if present.iter().any(|range| range.start == 0) {
        state.catalog_mut().require_tail_timeline_for(binding);
    }
}

pub(crate) async fn load_timeline(
    store: &PartialRangeStore,
    post: &PostId,
    total: u64,
    present: &[ByteRange],
) -> Option<MediaTimeline> {
    let ranges = metadata_ranges(total, present);
    let mut owned = Vec::with_capacity(ranges.len());
    for range in ranges {
        let bytes = store
            .read_range(post.as_str(), range.start..range.end)
            .await
            .ok()??;
        owned.push((range.start, bytes));
    }
    let segments: Vec<_> = owned
        .iter()
        .map(|(start, bytes)| MediaSegment::new(*start, bytes))
        .collect();
    parse_mp4_segments(&segments)
        .ok()
        .filter(|timeline| timeline.fits_within(total))
}

fn metadata_ranges(total: u64, present: &[ByteRange]) -> Vec<ByteRange> {
    let mut ranges = Vec::new();
    if let Some(prefix) = present.iter().find(|range| range.start == 0) {
        ranges.push(ByteRange::new(0, prefix.end.min(MAX_METADATA_SPAN)));
    }
    if let Some(tail) = present.iter().find(|range| range.end >= total) {
        let start = tail.start.max(total.saturating_sub(MAX_METADATA_SPAN));
        let candidate = ByteRange::new(start, total);
        if !ranges.contains(&candidate) {
            ranges.push(candidate);
        }
    }
    ranges
}
