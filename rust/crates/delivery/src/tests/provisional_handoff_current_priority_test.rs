use crate::manager::concurrency::planned_capacity;
use crate::manager::reconcile_warp;
use crate::tests::provisional_handoff_fixture::{cross_origin_handoff_state, CURRENT, NEXT};
use crate::tests::provisional_handoff_plan_fixture::plan;
use ghostr_engine::adaptive::{PlannerCommand, PreemptionAuthority};
use ghostr_engine::{ActionId, DataUsageLevel, PostId};
use std::collections::HashSet;

#[test]
fn capacity_contraction_yields_the_far_handoff_to_a_missing_current() {
    let (state, active) = cross_origin_handoff_state(DataUsageLevel::Conservative);
    let first = plan(state, &active, 2);
    let selected = first
        .warp
        .as_ref()
        .and_then(|warp| warp.selected.as_ref())
        .map(|action| &action.command);
    assert!(matches!(selected, Some(PlannerCommand::Cancel(id)) if *id == ActionId::new(1)));
    assert!(first.retained.contains(&ActionId::new(2)));
    assert!(!first.retained.contains(&ActionId::new(1)));

    let (state, active) = cross_origin_handoff_state(DataUsageLevel::Conservative);
    let second = plan(state, &active[1..], 2);
    let execution = reconcile_warp::execution(second);
    let current = execution
        .transfers
        .iter()
        .find(|transfer| transfer.request.chunk.post == PostId::new(CURRENT))
        .expect("current transfer after cancellation");
    assert_eq!(
        current.request.authority,
        PreemptionAuthority::PlaybackCritical
    );
    assert_eq!(execution.retained_posts, HashSet::from([PostId::new(NEXT)]));
    assert_eq!(
        planned_capacity(1, 2, &execution.transfers, &execution.retained_posts).total,
        2
    );
}
