//! Bounded background hydration of representation-fenced MP4 timing evidence.

use crate::manager::state::DeliveryState;
use crate::manager::DeliveryWorker;
use ghostr_engine::media_timeline::{normalize, MediaTimeline};
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::{ByteRange, PostId};
use ghostr_partial_store::partial_range_store::StoredMediaSnapshot;
use std::collections::HashMap;

mod application;
mod attempts;
mod coordinator;
mod evidence;
mod job;
#[cfg(test)]
mod load;
mod outcome;
mod parser;

#[cfg(test)]
pub(crate) use attempts::{TimelineAttemptDisposition, TimelineAttempts};
pub(crate) use coordinator::TimelineCoordinator;
pub(crate) use coordinator::TimelineSchedule;
pub(crate) use evidence::TimelineEvidence;
#[cfg(test)]
pub(crate) use load::load_timeline;
#[cfg(test)]
pub(crate) use outcome::TimelineIncomplete;
pub(crate) use outcome::{TimelineJobOutcome, TimelineRejection, TimelineResult, TimelineTerminal};
#[cfg(test)]
pub(crate) use parser::{TimelineInput, TimelineParse, TimelineParser};

const MAX_METADATA_SPAN: u64 = 4 * 1024 * 1024;

impl DeliveryWorker {
    pub(super) fn reconcile_timelines(
        &mut self,
        posts: &[PostId],
        snapshots: &HashMap<PostId, StoredMediaSnapshot>,
    ) {
        self.timelines
            .retain_active(&posts.iter().cloned().collect());
        self.apply_timeline_results(snapshots);
        self.schedule_timelines(posts, snapshots);
    }

    fn schedule_timelines(
        &mut self,
        posts: &[PostId],
        snapshots: &HashMap<PostId, StoredMediaSnapshot>,
    ) {
        for post in posts {
            self.schedule_timeline(post, snapshots.get(post));
        }
        self.timelines.dispatch(posts);
    }

    fn schedule_timeline(&mut self, post: &PostId, snapshot: Option<&StoredMediaSnapshot>) {
        let binding = self.state.catalog().binding(post);
        let evidence = binding
            .as_ref()
            .zip(snapshot)
            .and_then(|(binding, snapshot)| TimelineEvidence::from_snapshot(binding, snapshot));
        let Some(evidence) = evidence else {
            self.invalidate_timeline(post, binding.as_ref());
            return;
        };
        let has_timeline = self.catalog_has_timeline(post);
        let preserve = self.timelines.preserves_publication(
            post,
            snapshot.expect("evidence has a snapshot"),
            has_timeline,
        );
        if self.timelines.schedule(post.clone(), evidence) != TimelineSchedule::Current && !preserve
        {
            self.clear_timeline(
                post,
                binding.as_ref().expect("timeline evidence has a binding"),
            );
        }
    }

    fn invalidate_timeline(&mut self, post: &PostId, binding: Option<&RepresentationBinding>) {
        self.timelines.invalidate(post);
        if let Some(binding) = binding {
            self.state.catalog_mut().clear_timeline_for(binding);
        }
    }
}

pub(crate) fn install_timeline(
    state: &mut DeliveryState,
    binding: &RepresentationBinding,
    timeline: MediaTimeline,
) -> bool {
    state.catalog_mut().learn_timeline_observation_for(
        binding,
        timeline,
        crate::manager::time::unix_time_ms(),
    )
}

fn engine_ranges(ranges: &[std::ops::Range<u64>]) -> Vec<ByteRange> {
    ranges
        .iter()
        .map(|range| ByteRange::new(range.start, range.end))
        .collect()
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
    normalize(ranges)
}
