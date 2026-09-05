use super::{
    engine_ranges, install_timeline, TimelineEvidence, TimelineJobOutcome, TimelineRejection,
    TimelineTerminal,
};
use crate::evaluation::IntegrityMetricEvent;
use crate::manager::state::DeliveryState;
use crate::manager::DeliveryWorker;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::{ByteRange, PostId};
use ghostr_partial_store::partial_range_store::StoredMediaSnapshot;
use std::collections::HashMap;

impl DeliveryWorker {
    pub(super) fn apply_timeline_results(
        &mut self,
        snapshots: &HashMap<PostId, StoredMediaSnapshot>,
    ) {
        for result in self.timelines.take_completed() {
            let post = result.post().clone();
            let current = current_evidence(&self.state, &self.timelines, &post, snapshots);
            let Some(outcome) = self.timelines.validate(result, current.as_ref()) else {
                continue;
            };
            let (Some(evidence), Some(snapshot)) = (current, snapshots.get(&post)) else {
                continue;
            };
            self.apply_timeline_outcome(&post, snapshot, &evidence, outcome);
        }
    }

    fn apply_timeline_outcome(
        &mut self,
        post: &PostId,
        snapshot: &StoredMediaSnapshot,
        evidence: &TimelineEvidence,
        outcome: TimelineJobOutcome,
    ) {
        match outcome {
            TimelineJobOutcome::Terminal(TimelineTerminal::Ready(timeline)) => {
                if install_timeline(
                    &mut self.state,
                    evidence.binding(),
                    *timeline,
                    evidence
                        .source
                        .as_ref()
                        .map(|(identity, _)| identity.clone()),
                ) {
                    self.timelines
                        .publish_installed(post.clone(), evidence.clone());
                }
            }
            TimelineJobOutcome::Terminal(TimelineTerminal::Incomplete(reason)) => {
                log::debug!(
                    "Timeline evidence incomplete for {}: {reason:?}",
                    post.as_str()
                );
                self.apply_incomplete_timeline(post, snapshot, evidence);
            }
            TimelineJobOutcome::Terminal(TimelineTerminal::Rejected(reason)) => {
                log::debug!(
                    "Timeline evidence rejected for {}: {reason:?}",
                    post.as_str()
                );
                self.record_timeline_rejection(reason);
                self.clear_timeline(post, evidence.binding());
            }
            TimelineJobOutcome::Retryable(_) | TimelineJobOutcome::Superseded => {}
        }
    }

    fn apply_incomplete_timeline(
        &mut self,
        post: &PostId,
        snapshot: &StoredMediaSnapshot,
        evidence: &TimelineEvidence,
    ) {
        let has_timeline = self.catalog_has_timeline(post);
        if self
            .timelines
            .preserves_publication(post, snapshot, has_timeline)
        {
            if !has_timeline {
                self.timelines
                    .publish_tail_needed(post.clone(), evidence.clone());
            }
            return;
        }
        self.clear_timeline(post, evidence.binding());
        let present = engine_ranges(snapshot.ranges());
        if require_tail_if_started(&mut self.state, evidence.binding(), &present) {
            self.timelines
                .publish_tail_needed(post.clone(), evidence.clone());
        }
    }

    pub(super) fn clear_timeline(&mut self, post: &PostId, binding: &RepresentationBinding) {
        self.timelines.forget_publication(post);
        self.state.catalog_mut().clear_timeline_for(binding);
    }

    pub(super) fn catalog_has_timeline(&self, post: &PostId) -> bool {
        self.state
            .catalog()
            .lookup(post)
            .is_some_and(|entry| entry.timeline().is_some())
    }

    fn record_timeline_rejection(&self, reason: TimelineRejection) {
        let event = match reason {
            TimelineRejection::OutOfBounds => IntegrityMetricEvent::IncorrectRangeSplicePrevented,
            TimelineRejection::ResourceLimit => IntegrityMetricEvent::ParserLimitRejection,
            TimelineRejection::Malformed => IntegrityMetricEvent::MetadataCalibrationError,
            TimelineRejection::Unsupported => return,
        };
        self.commands.evaluation().integrity(event);
    }
}

fn current_evidence(
    state: &DeliveryState,
    coordinator: &super::TimelineCoordinator,
    post: &PostId,
    snapshots: &HashMap<PostId, StoredMediaSnapshot>,
) -> Option<TimelineEvidence> {
    let binding = state.catalog().binding(post)?;
    coordinator.evidence(&binding, snapshots.get(post)?)
}

fn require_tail_if_started(
    state: &mut DeliveryState,
    binding: &RepresentationBinding,
    present: &[ByteRange],
) -> bool {
    present.iter().any(|range| range.start == 0)
        && state.catalog_mut().require_tail_timeline_for(binding)
}
