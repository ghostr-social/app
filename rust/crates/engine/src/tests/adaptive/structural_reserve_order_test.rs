use crate::adaptive::{AdaptivePlayabilityPolicy, NextReserveEvidence};
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

#[test]
fn a_distant_ready_rescue_does_not_suppress_immediate_next_preparation() {
    let mut input = snapshot(3, 20_000_000, 20_000, 0);
    let ready = input.candidates[2]
        .startup
        .as_ref()
        .expect("valid test fixture")
        .ranges()[0];
    input.candidates[2].present = vec![ready];

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(matches!(
        plan.next_reserve,
        NextReserveEvidence::Granted { post, .. } if post == PostId::new("p1")
    ));
}
