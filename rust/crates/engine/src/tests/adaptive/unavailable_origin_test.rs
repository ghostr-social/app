use crate::adaptive::AdaptivePlayabilityPolicy;
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

#[test]
fn a_candidate_without_an_available_origin_is_not_admitted() {
    let mut input = snapshot(3, 20_000_000, 20_000, 2);
    input.candidates[1].origins[0].available = false;

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(plan
        .allocations
        .iter()
        .all(|work| work.post != PostId::new("p1")));
}
