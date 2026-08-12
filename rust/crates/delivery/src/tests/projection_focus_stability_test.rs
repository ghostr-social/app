use super::candidate_catalog_fixture::candidate;
use crate::manager::state::DeliveryState;
use ghostr_engine::{DataUsageLevel, EngineParams, PostId};

/// Before the UI supplies focus, the projected current post must stay
/// pinned while discovery keeps delivering newer candidates. The feed
/// accumulates rows (its top post is the first one served), so
/// re-aiming at every arrival restarts the first video's startup work.
#[test]
fn projected_current_post_survives_newer_discoveries() {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);

    state.apply_candidate(candidate("first", 1));
    assert_eq!(state.focus().current(), Some(&PostId::new("first")));

    state.apply_candidate(candidate("second", 2));
    state.apply_candidate(candidate("third", 3));

    assert_eq!(state.focus().current(), Some(&PostId::new("first")));
}
