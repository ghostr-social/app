use crate::adaptive::{AdaptivePlayabilityPolicy, OriginHealth};
use crate::tests::adaptive_support::{healthy_origin, snapshot};
use crate::PostId;

#[test]
fn a_failed_primary_reallocates_the_candidate_to_its_healthy_alternative() {
    let policy = AdaptivePlayabilityPolicy;
    let mut input = snapshot(3, 20_000_000, 20_000, 2);
    input.candidates[1].origins = vec![failed_primary(), healthy_mirror()];

    let plan = policy.plan(&input);
    let work = plan
        .allocations
        .iter()
        .find(|work| work.post == PostId::new("p1"))
        .expect("candidate remains admitted through its mirror");

    assert_eq!(work.source, "mirror");
}

fn failed_primary() -> OriginHealth {
    let mut origin = healthy_origin("primary", 100_000_000, 10);
    origin.failure_bps = 10_000;
    origin
}

fn healthy_mirror() -> OriginHealth {
    healthy_origin("mirror", 5_000_000, 100)
}
