use super::candidate_catalog_fixture::candidate;
use crate::delivery_events::{DeliveryFocus, FocusItem};
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::CurrentAuthority;
use ghostr_engine::{DataUsageLevel, EngineParams, PostId};

#[test]
fn canonical_roster_retargets_a_different_first_relay_arrival() {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    state.apply_candidate(candidate("first-arrival", 1));
    state.apply_candidate(candidate("canonical-newest", 2));
    let items = ["canonical-newest", "first-arrival"]
        .into_iter()
        .map(item)
        .collect();

    assert!(state.apply_focus(DeliveryFocus::compatibility(items, 0, 0), 3));

    assert_eq!(
        state.focus().current(),
        Some(&PostId::new("canonical-newest"))
    );
    assert_eq!(state.current_authority(), CurrentAuthority::Canonical);
}

fn item(post: &str) -> FocusItem {
    let candidate = candidate(post, 1);
    FocusItem {
        post: candidate.post,
        meta: candidate.meta,
    }
}
