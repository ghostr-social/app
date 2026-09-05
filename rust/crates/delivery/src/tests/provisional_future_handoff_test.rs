use crate::tests::provisional_handoff_fixture::terminal::{
    terminal_current_handoff_state, terminal_future_handoff_state,
};
use crate::tests::provisional_handoff_fixture::{
    acknowledged_full_roster_handoff_state, handoff_state, CURRENT,
};
use crate::tests::provisional_handoff_plan_fixture::{generated_cancels, plan};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::{ActionId, DataUsageLevel, PostId};
use std::collections::HashSet;

#[test]
fn partial_canonical_roster_yields_far_work_to_current_with_two_slots() {
    let (state, active) = handoff_state(DataUsageLevel::Aggressive);
    let work = plan(state, &active, 3);

    assert_eq!(generated_cancels(&work), HashSet::from([ActionId::new(1)]));
    assert_eq!(work.retained, HashSet::from([ActionId::new(2)]));
}

#[test]
fn current_continuation_preempts_historical_prefixes_at_the_two_request_cap() {
    let (state, active) = acknowledged_full_roster_handoff_state();
    let work = plan(state, &active, 3);

    assert_eq!(
        generated_cancels(&work),
        HashSet::from([ActionId::new(1), ActionId::new(2)])
    );
    // One cancellation is committed per pass; the remaining one is reconsidered
    // after its peer releases the request slot.
    assert_eq!(
        work.retained,
        HashSet::from([ActionId::new(1), ActionId::new(3)])
    );
}

#[test]
fn terminal_current_ack_leaves_one_handoff_slot_after_current_refill() {
    let (state, active) = terminal_current_handoff_state();
    let work = plan(state, &active, 3);

    assert!(work.plan.allocations.iter().any(|allocation| {
        allocation.post == PostId::new(CURRENT)
            && allocation.request.requested_bytes().start == 65_536
            && allocation.authority == PreemptionAuthority::PlaybackCritical
    }));
    assert_eq!(generated_cancels(&work), HashSet::from([ActionId::new(1)]));
    assert!(work.retained.contains(&ActionId::new(2)));
    assert!(work.retained.contains(&ActionId::new(3)));
}

#[test]
fn terminal_future_ack_is_not_cancelled_or_charged_as_live_io() {
    let (state, active) = terminal_future_handoff_state();
    let work = plan(state, &active, 3);

    assert_eq!(generated_cancels(&work), HashSet::from([ActionId::new(2)]));
    assert_eq!(
        work.retained,
        HashSet::from([ActionId::new(1), ActionId::new(3)])
    );
}
