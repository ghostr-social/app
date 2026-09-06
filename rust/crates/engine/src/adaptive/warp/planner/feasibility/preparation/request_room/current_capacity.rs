use super::WarpPlannerInput;
use crate::RequestAuthority;

/// A second current request cannot displace the same current body's origin slot.
pub(super) fn owns_origin(input: &WarpPlannerInput<'_>, source: &str) -> bool {
    let Some(authority) = RequestAuthority::from_url(source) else {
        return false;
    };
    input
        .snapshot
        .candidates
        .iter()
        .filter(|candidate| candidate.post == input.snapshot.playback.current)
        .flat_map(|candidate| &candidate.in_flight)
        .filter(|active| active.identity_current && !active.cancelling)
        .filter(|active| RequestAuthority::from_url(&active.source).as_ref() == Some(&authority))
        .count()
        >= usize::from(input.context.limits.per_origin_requests.max(1))
}
