use super::super::allocation::AllocationSpec;
use super::super::builder::{Builder, TransferInput};
use super::super::{GeneratedAction, PlannerCommand};
use crate::adaptive::{
    ActionKind, CandidateSnapshot, MediaLayout, RetrievalRequest, BOOTSTRAP_DIRECT_FETCH_BYTES,
    REQUEST_SLICE_BYTES,
};

impl Builder<'_> {
    pub(super) fn add_whole(&mut self, candidate: &CandidateSnapshot) -> Option<u16> {
        let maximum = self.maximum(candidate)?;
        if whole_is_owned(candidate)
            || candidate.finalized
            || crate::adaptive::resources::fully_stored(candidate)
        {
            return None;
        }
        let kind = ActionKind::FetchWhole {
            maximum_bytes: maximum,
        };
        let source = self.admitted_request_source(candidate, &kind)?;
        if single_slice_range_dominates_whole(self, candidate, source) {
            return None;
        }
        if self.contains(candidate, &kind) {
            return self.action_id(candidate, |item| {
                matches!(item, ActionKind::FetchWhole { .. })
            });
        }
        let spec = candidate.total_bytes.map_or_else(
            || AllocationSpec::unknown_whole_probe(maximum, source),
            |_| AllocationSpec::whole(maximum, source, candidate.duration_ms),
        );
        let allocation = self.allocation(candidate, spec);
        self.push_transfer(candidate, TransferInput::delivery(kind, allocation, &[]))
    }

    fn maximum(&self, candidate: &CandidateSnapshot) -> Option<u64> {
        candidate.total_bytes.or_else(|| {
            (candidate.layout == MediaLayout::RequiresCompleteFile)
                .then(|| self.next_unknown_cap(candidate))
                .flatten()
        })
    }

    fn next_unknown_cap(&self, candidate: &CandidateSnapshot) -> Option<u64> {
        let exhaustion = self
            .context
            .candidate(&candidate.post)
            .and_then(|item| item.whole_body_exhaustion);
        let Some(exhaustion) = exhaustion else {
            return Some(BOOTSTRAP_DIRECT_FETCH_BYTES);
        };
        let minimum = exhaustion
            .observed_bytes()
            .checked_add(REQUEST_SLICE_BYTES)?;
        let target = exhaustion.maximum_bytes().saturating_mul(4).max(minimum);
        let bounded = target.min(self.context.limits.network_burst_bytes);
        if bounded >= minimum {
            return Some(bounded);
        }
        Some(target)
    }
}

fn whole_is_owned(candidate: &CandidateSnapshot) -> bool {
    candidate
        .in_flight
        .iter()
        .any(|active| active.identity_current)
}

/// Prune only when the emitted request and retained bytes complete the object.
fn single_slice_range_dominates_whole(
    builder: &Builder<'_>,
    candidate: &CandidateSnapshot,
    source: &str,
) -> bool {
    if candidate.layout == MediaLayout::RequiresCompleteFile
        || candidate.present.is_empty()
        || candidate.total_bytes.is_none()
        || candidate.direct_playback_blocked
        || builder.direct_playback_blocked(candidate)
    {
        return false;
    }
    builder
        .actions
        .iter()
        .any(|action| completing_range(action, candidate, source))
}

fn completing_range(action: &GeneratedAction, candidate: &CandidateSnapshot, source: &str) -> bool {
    if action.node.post != candidate.post {
        return false;
    }
    let PlannerCommand::Transfer(allocation) = &action.command else {
        return false;
    };
    let RetrievalRequest::FetchRange { bytes, .. } = allocation.request else {
        return false;
    };
    let Some(total) = candidate.total_bytes else {
        return false;
    };
    let mut coverage = candidate.present.clone();
    coverage.push(bytes);
    allocation.source == source
        && crate::adaptive::ranges::uncovered_bytes(crate::ByteRange::new(0, total), &coverage) == 0
}
