use crate::adaptive::{AdaptivePlayabilityPolicy, MediaLayout};
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

#[test]
fn emergency_does_not_bypass_an_unplannable_current_video() {
    let mut input = snapshot(2, 20_000_000, 0, 0);
    input.candidates[0].layout = MediaLayout::Unknown;

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(plan.allocations.is_empty(), "{plan:#?}");
    assert!(plan
        .retained
        .iter()
        .all(|work| work.post == PostId::new("p0")));
}
