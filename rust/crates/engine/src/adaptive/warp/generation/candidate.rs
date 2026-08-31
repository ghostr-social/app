use super::allocation::{classify, resources, AllocationSpec};
use super::builder::{Builder, NodeInput, TransferInput};
use super::{GeneratedAction, PlannerCommand};
use crate::adaptive::{
    ActionKind, Allocation, CandidateSnapshot, MediaLayout, PlayabilitySnapshot,
    PreemptionAuthority,
};
use crate::ByteRange;

mod transform;
mod whole;

impl Builder<'_> {
    pub(super) fn add_candidate(&mut self, candidate: &CandidateSnapshot) {
        self.add_head(candidate);
        if has_active_whole(candidate) {
            self.add_active(candidate);
            return;
        }
        self.add_base(candidate);
        if candidate.layout != MediaLayout::RequiresCompleteFile {
            self.add_prefix(candidate);
            let prefix = self.action_id(candidate, |kind| matches!(kind, ActionKind::Prefix(_)));
            self.add_tail(candidate, prefix);
            self.add_continuation(candidate);
        }
        let whole = self.add_whole(candidate);
        self.add_cache_upgrade(candidate);
        self.add_transform(candidate, whole);
        self.add_active(candidate);
    }

    fn add_head(&mut self, candidate: &CandidateSnapshot) {
        let head_suppressed = self
            .context
            .candidate(&candidate.post)
            .is_some_and(|item| item.head_probe != super::super::HeadProbeHistory::Unobserved);
        let current = candidate.post == self.snapshot.playback.current;
        if !candidate.needs_bootstrap() || current || head_suppressed {
            return;
        }
        let kind = ActionKind::Head;
        let Some(source) = self.optional_exploration_source(candidate, &kind) else {
            return;
        };
        let prediction = self.prediction(candidate, &kind, source);
        let input = NodeInput::new(kind.clone(), source, prediction, &[]).optional_exploration();
        let mut node = self.node(candidate, input);
        node.resources = resources(&kind);
        self.actions.push(GeneratedAction {
            node,
            command: PlannerCommand::ProbeHead {
                post: candidate.post.clone(),
                source: source.to_owned(),
                authority: super::allocation::authority(candidate, self.snapshot, self.base.mode),
            },
        });
    }

    fn add_base(&mut self, candidate: &CandidateSnapshot) {
        if !self.permits_request(candidate) {
            return;
        }
        let allocations: Vec<_> = self
            .base
            .allocations
            .iter()
            .filter(|item| item.post == candidate.post)
            .cloned()
            .collect();
        for allocation in allocations {
            let allocation = self.normalize_playability(candidate, allocation);
            let input = TransferInput::delivery(classify(&allocation), allocation, &[]);
            let _ = self.push_transfer(candidate, input);
        }
    }

    fn add_prefix(&mut self, candidate: &CandidateSnapshot) {
        if !candidate.needs_bootstrap() {
            return;
        }
        let Some(missing) = super::super::super::ranges::missing(candidate)
            .into_iter()
            .find(|item| item.bytes.start < crate::adaptive::MEDIA_BOOTSTRAP_PROBE_BYTES)
        else {
            return;
        };
        let range = bounded_range(missing.bytes, crate::adaptive::MEDIA_BOOTSTRAP_PROBE_BYTES);
        let kind = ActionKind::Prefix(range);
        let Some(source) = self.optional_exploration_source(candidate, &kind) else {
            return;
        };
        if self.contains(candidate, &kind) {
            return;
        }
        let mut allocation = self.allocation(candidate, AllocationSpec::range(range, source, 0));
        protect_current_prefix(self.snapshot, candidate, &mut allocation);
        let input = TransferInput::optional_exploration(kind, allocation, &[]);
        let _ = self.push_transfer(candidate, input);
    }

    fn add_tail(&mut self, candidate: &CandidateSnapshot, prefix: Option<u16>) {
        let Some(probe) = candidate.timeline_probe else {
            return;
        };
        let Some(missing) = super::super::super::ranges::missing_playable(candidate, probe)
            .into_iter()
            .next()
        else {
            return;
        };
        let range = bounded_range(missing.bytes, self.snapshot.request_slice_bytes);
        let kind = ActionKind::Tail(range);
        let Some(source) = self.optional_exploration_source(candidate, &kind) else {
            return;
        };
        if self.contains(candidate, &kind) {
            return;
        }
        let allocation = self.allocation(candidate, AllocationSpec::range(range, source, 0));
        let dependencies: Vec<_> = prefix.into_iter().collect();
        let input = TransferInput::optional_exploration(kind, allocation, &dependencies);
        let _ = self.push_transfer(candidate, input);
    }

    fn add_continuation(&mut self, candidate: &CandidateSnapshot) {
        let Some(playable) = super::super::super::ranges::missing(candidate)
            .into_iter()
            .next()
        else {
            return;
        };
        let range = bounded_range(playable.bytes, self.snapshot.request_slice_bytes);
        let kind = ActionKind::FetchRange(range);
        let Some(source) = self.admitted_request_source(candidate, &kind) else {
            return;
        };
        if self.contains_promotable_range_transfer(candidate, source, range)
            || self.contains(candidate, &kind)
        {
            return;
        }
        let allocation = self.allocation(
            candidate,
            AllocationSpec::range(range, source, playable.playable_ms),
        );
        let input = TransferInput::delivery(kind, allocation, &[]);
        let _ = self.push_transfer(candidate, input);
    }

    fn add_cache_upgrade(&mut self, candidate: &CandidateSnapshot) {
        if candidate.layout == MediaLayout::RequiresCompleteFile || candidate.present.is_empty() {
            return;
        }
        let Some(missing) = super::super::super::ranges::missing(candidate)
            .into_iter()
            .next()
        else {
            return;
        };
        let range = bounded_range(missing.bytes, self.snapshot.request_slice_bytes);
        let kind = ActionKind::CacheUpgrade(range);
        let Some(source) = self.admitted_request_source(candidate, &kind) else {
            return;
        };
        if self.contains_promotable_range_transfer(candidate, source, range) {
            return;
        }
        let allocation = self.allocation(candidate, AllocationSpec::cache(range, source));
        let input = TransferInput::delivery(kind, allocation, &[]);
        let _ = self.push_transfer(candidate, input);
    }
}

fn protect_current_prefix(
    snapshot: &PlayabilitySnapshot,
    candidate: &CandidateSnapshot,
    allocation: &mut Allocation,
) {
    if candidate.post == snapshot.playback.current {
        allocation.authority = PreemptionAuthority::PlaybackCritical;
    }
}

fn bounded_range(range: ByteRange, maximum: u64) -> ByteRange {
    let end = range.start.saturating_add(maximum).min(range.end);
    ByteRange::new(range.start, end)
}

fn has_active_whole(candidate: &CandidateSnapshot) -> bool {
    candidate.in_flight.iter().any(|active| {
        active.identity_current
            && matches!(
                active.request,
                crate::adaptive::RetrievalRequest::FetchWhole { .. }
            )
    })
}
