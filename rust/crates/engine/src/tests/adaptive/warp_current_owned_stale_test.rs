use super::warp_current_owned_fixture::{history_admitted, state};

#[test]
fn stale_current_response_does_not_remove_its_replacement_reservation() {
    let mut state = state();
    state.candidates[0].in_flight[0].identity_current = false;

    assert!(
        !history_admitted(&state, 1),
        "stale identity still needs a current replacement"
    );
}
