use crate::adaptive::{AdaptivePlayabilityPolicy, NextReserveEvidence, NextReserveInfeasibility};
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

#[test]
fn an_unavailable_immediate_next_origin_has_explicit_infeasibility_evidence() {
    let mut input = snapshot(2, 700_000, 0, 2);
    input.candidates[1].origins[0].available = false;

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert_eq!(
        plan.next_reserve,
        NextReserveEvidence::Infeasible {
            post: PostId::new("p1"),
            reason: NextReserveInfeasibility::NoLiveOrigin,
        },
    );
}
