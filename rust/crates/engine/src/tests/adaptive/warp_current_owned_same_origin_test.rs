use super::warp_current_owned_fixture::{history_admitted, state};

#[test]
fn spare_global_capacity_cannot_bypass_the_current_origin_limit() {
    let mut state = state();
    state.candidates[1].origins[0].source = "https://origin.example/previous.mp4".into();

    assert!(
        !history_admitted(&state, 1),
        "the origin hard limit remains enforced"
    );
}
