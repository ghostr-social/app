use crate::adaptive::{AdaptivePlayabilityPolicy, AllocationReason, MediaLayout};
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

#[test]
fn unknown_current_receives_only_one_bounded_bootstrap_range() {
    let mut input = snapshot(2, 20_000_000, 0, 0);
    input.candidates[0].layout = MediaLayout::Unknown;

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    let current: Vec<_> = plan
        .allocations
        .iter()
        .filter(|work| work.post == PostId::new("p0"))
        .collect();
    assert_eq!(current.len(), 1, "{plan:#?}");
    assert_eq!(current[0].reason, AllocationReason::MediaBootstrap);
    assert!(current[0].request.requested_bytes().len() <= input.request_slice_bytes);
    assert!(plan
        .retained
        .iter()
        .all(|work| work.post == PostId::new("p0")));
}
