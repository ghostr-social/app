use super::warp_current_owned_fixture::{history_admitted, state};

#[test]
fn current_reserves_its_second_request_when_its_origin_can_accept_it() {
    assert!(
        !history_admitted(&state(), 2),
        "feasible current work retains the spare request"
    );
}
