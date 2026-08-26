use crate::adaptive::{AdaptivePlayabilityPolicy, AllocationReason, ViewProbability};
use crate::tests::adaptive_support::{frontier, snapshot};
use crate::PostId;

#[test]
fn scarce_capacity_goes_to_the_more_likely_viewing_choice() {
    let policy = AdaptivePlayabilityPolicy;
    let mut input = snapshot(3, 1_100_000, 20_000, 2);
    input.candidates[1].view_probability = ViewProbability::new(0.05).expect("valid test fixture");
    input.candidates[2].view_probability = ViewProbability::new(0.95).expect("valid test fixture");

    let plan = policy.plan(&input);

    assert!(frontier(&plan).contains(&PostId::new("p2")), "{plan:#?}");
    assert!(frontier(&plan).contains(&PostId::new("p1")), "{plan:#?}");
    let p1 = plan
        .allocations
        .iter()
        .find(|work| work.post == PostId::new("p1"))
        .expect("immediate-next initialization");
    assert_eq!(p1.reason, AllocationReason::NextStartability);
}
