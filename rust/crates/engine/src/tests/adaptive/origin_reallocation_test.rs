use crate::adaptive::{AdaptivePlayabilityPolicy, OriginHealth};
use crate::tests::adaptive_support::{healthy_origin, snapshot};
use crate::tests::support::set_reliable_total_bytes_for_source;
use crate::PostId;

const PRIMARY: &str = "https://primary.example/media";
const MIRROR: &str = "https://mirror.example/media";

#[test]
fn a_failed_primary_reallocates_the_candidate_to_its_healthy_alternative() {
    let policy = AdaptivePlayabilityPolicy;
    let mut input = snapshot(3, 20_000_000, 20_000, 2);
    input.candidates[1].origins = vec![failed_primary(), healthy_mirror()];
    let total = input.candidates[1].total_bytes.expect("known total");
    let observed_at_ms = input.observed_at_ms;
    set_reliable_total_bytes_for_source(&mut input.candidates[1], total, observed_at_ms, MIRROR);

    let plan = policy.plan(&input);
    let work = plan
        .allocations
        .iter()
        .find(|work| work.post == PostId::new("p1"))
        .expect("candidate remains admitted through its mirror");

    assert_eq!(work.source, MIRROR);
}

fn failed_primary() -> OriginHealth {
    let mut origin = healthy_origin(PRIMARY, 100_000_000, 10);
    origin.failure_bps = 10_000;
    origin
}

fn healthy_mirror() -> OriginHealth {
    healthy_origin(MIRROR, 5_000_000, 100)
}
