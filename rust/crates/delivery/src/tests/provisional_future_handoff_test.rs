use crate::manager::concurrency::planned_capacity;
use crate::manager::reconcile_warp;
use crate::tests::provisional_handoff_fixture::{
    acknowledged_full_roster_handoff_state, handoff_state, CURRENT, NEXT, THIRD,
};
use crate::tests::provisional_handoff_fixture::terminal::{
    terminal_current_handoff_state, terminal_future_handoff_state,
};
use crate::tests::provisional_handoff_plan_fixture::{generated_cancels, plan};
use ghostr_engine::adaptive::{PlannerCommand, PreemptionAuthority};
use ghostr_engine::{ActionId, DataUsageLevel, PostId};
use std::collections::HashSet;

#[test]
fn bounded_provisional_futures_survive_the_first_partial_canonical_roster() {
    let (state, active) = handoff_state(DataUsageLevel::Aggressive);

    let work = plan(state, &active, 3);
    let warp = work.warp.as_ref().expect("WARP decision");
    assert!(warp
        .generated
        .actions
        .iter()
        .all(|action| !matches!(action.command, PlannerCommand::Cancel(_))));
    assert!(work.retained.contains(&ActionId::new(1)));
    assert!(work.retained.contains(&ActionId::new(2)));
    let execution = reconcile_warp::execution(work);
    let current = execution
        .transfers
        .iter()
        .find(|transfer| transfer.request.chunk.post == PostId::new(CURRENT))
        .expect("missing current transfer");
    assert_eq!(
        current.request.authority,
        PreemptionAuthority::PlaybackCritical
    );
    assert_eq!(
        execution.retained_posts,
        HashSet::from([PostId::new(NEXT), PostId::new(THIRD)])
    );
    assert_eq!(
        planned_capacity(1, 4, &execution.transfers, &execution.retained_posts).total,
        3
    );
}

#[test]
fn canonical_acknowledgement_keeps_pre_transition_prefixes() {
    let (state, active) = acknowledged_full_roster_handoff_state();

    let work = plan(state, &active, 3);

    assert!(generated_cancels(&work).is_empty(), "{:#?}", work.warp);
    assert_eq!(
        work.retained,
        HashSet::from([ActionId::new(1), ActionId::new(2), ActionId::new(3)])
    );
}

#[test]
fn terminal_current_ack_does_not_displace_handoff_prefixes() {
    let (state, active) = terminal_current_handoff_state();

    let work = plan(state, &active, 3);

    assert!(work.plan.allocations.iter().any(|allocation| {
        allocation.post == PostId::new(CURRENT)
            && allocation.request.requested_bytes().start == 65_536
            && allocation.authority == PreemptionAuthority::PlaybackCritical
    }));
    assert!(generated_cancels(&work).is_empty(), "{:#?}", work.warp);
    assert!(work.retained.contains(&ActionId::new(1)));
    assert!(work.retained.contains(&ActionId::new(2)));
}

#[test]
fn terminal_future_ack_is_not_cancelled_or_charged_as_live_io() {
    let (state, active) = terminal_future_handoff_state();

    let work = plan(state, &active, 3);

    assert!(generated_cancels(&work).is_empty(), "{:#?}", work.warp);
    assert!(work.retained.contains(&ActionId::new(1)));
    assert!(work.retained.contains(&ActionId::new(2)));
}
