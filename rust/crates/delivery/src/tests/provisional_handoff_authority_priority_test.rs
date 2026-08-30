use crate::manager::concurrency::planned_capacity;
use crate::manager::reconcile_warp;
use crate::tests::provisional_handoff_fixture::{handoff_state, CURRENT, NEXT};
use crate::tests::provisional_handoff_plan_fixture::plan;
use ghostr_engine::adaptive::{PlannerCommand, PreemptionAuthority};
use ghostr_engine::{ActionId, DataUsageLevel, PostId};
use std::collections::HashSet;

#[test]
fn authority_saturation_cancels_the_far_handoff_before_current_refill() {
    let (state, active) = handoff_state(DataUsageLevel::Balanced);
    let first = plan(state, &active, 2);
    let selected = first
        .warp
        .as_ref()
        .and_then(|warp| warp.selected.as_ref())
        .map(|action| &action.command);
    assert!(matches!(selected, Some(PlannerCommand::Cancel(id)) if *id == ActionId::new(1)));
    assert!(first.retained.contains(&ActionId::new(2)));
    assert!(!first.retained.contains(&ActionId::new(1)));

    let (state, active) = handoff_state(DataUsageLevel::Balanced);
    let second = plan(state, &active[1..], 2);
    let execution = reconcile_warp::execution(second);
    let current = execution
        .transfers
        .iter()
        .find(|transfer| transfer.request.chunk.post == PostId::new(CURRENT))
        .expect("current transfer after authority release");
    assert_eq!(
        current.request.authority,
        PreemptionAuthority::PlaybackCritical
    );
    assert_eq!(execution.retained_posts, HashSet::from([PostId::new(NEXT)]));
    assert_eq!(
        planned_capacity(1, 3, &execution.transfers, &execution.retained_posts).total,
        2
    );
}
