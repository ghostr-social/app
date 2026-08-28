use super::super::allocation::AllocationSpec;
use super::super::builder::{Builder, TransferInput};
use crate::adaptive::{
    ActionKind, CandidateSnapshot, MediaLayout, BOOTSTRAP_DIRECT_FETCH_BYTES, REQUEST_SLICE_BYTES,
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
