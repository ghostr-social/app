use super::super::{CandidateSnapshot, HlsBootstrapState, PlayabilitySnapshot};

pub(super) fn allows(snapshot: &PlayabilitySnapshot, candidate: &CandidateSnapshot) -> bool {
    !snapshot.hls_candidates.iter().any(|hls| {
        hls.feed_offset.value() > 0
            && hls.feed_offset.value() < candidate.feed_offset.value()
            && unprotected(hls)
    })
}

fn unprotected(candidate: &super::super::HlsCandidateSnapshot) -> bool {
    matches!(candidate.state, HlsBootstrapState::Pending { .. })
        || matches!(
            candidate.state,
            HlsBootstrapState::Active {
                cancelling: true,
                ..
            }
        )
}
