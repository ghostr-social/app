use crate::adaptive::{
    AdaptivePlayabilityPolicy, AllocationReason, CurrentAuthority, MediaLayout, PreemptionAuthority,
};
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

#[test]
fn provisional_current_bootstrap_is_bounded_and_never_playback_critical() {
    let mut input = snapshot(1, 20_000_000, 0, 0);
    input.playback.authority = CurrentAuthority::Provisional;
    input.candidates[0].total_bytes = None;
    input.candidates[0].layout = MediaLayout::Unknown;

    let plan = AdaptivePlayabilityPolicy.plan(&input);
    let current: Vec<_> = plan
        .allocations
        .iter()
        .filter(|work| work.post == PostId::new("p0"))
        .collect();

    assert_eq!(current.len(), 1, "{plan:#?}");
    assert_eq!(current[0].reason, AllocationReason::MediaBootstrap);
    assert_eq!(current[0].authority, PreemptionAuthority::Speculative);
    assert!(current[0].request.requested_bytes().len() <= input.request_slice_bytes);
}
