use crate::adaptive::AdaptivePlayabilityPolicy;
use crate::tests::adaptive_support::snapshot;
use crate::{ByteRange, PostId};

#[test]
fn low_probability_evicted_ranges_wait_while_a_likely_transition_can_reacquire() {
    let mut input = snapshot(8, 20_000_000, 20_000, 20);
    let range = ByteRange::new(0, 250_000);
    assert!(allocated(
        &AdaptivePlayabilityPolicy.plan(&input),
        "p3",
        range
    ));
    input.candidates[1].recently_evicted.push(range);
    input.candidates[3].recently_evicted.push(range);

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(allocated(&plan, "p1", range), "{plan:#?}");
    assert!(!allocated(&plan, "p3", range), "{plan:#?}");
}

fn allocated(plan: &crate::adaptive::AllocationPlan, post: &str, range: ByteRange) -> bool {
    plan.allocations.iter().any(|work| {
        let requested = work.request.requested_bytes();
        work.post == PostId::new(post) && requested.start < range.end && range.start < requested.end
    })
}
