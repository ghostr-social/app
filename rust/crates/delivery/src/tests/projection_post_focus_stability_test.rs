use super::candidate_catalog_fixture::candidate;
use crate::delivery_events::{DeliveryFocus, FocusItem};
use crate::manager::state::DeliveryState;
use ghostr_engine::{DataUsageLevel, EngineParams, PostId};

#[test]
fn later_relay_arrival_cannot_replace_canonical_focus() {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    state.apply_candidate(candidate("relay", 1));
    let focused = candidate("canonical", 2);
    let item = FocusItem {
        post: focused.post,
        meta: focused.meta,
    };
    assert!(state.apply_focus(DeliveryFocus::compatibility(vec![item], 0, 0), 2));

    state.apply_candidate(candidate("late-newer", 3));

    assert_eq!(state.focus().current(), Some(&PostId::new("canonical")));
}
