use crate::tests::adaptive_plan_assertions::allocated_posts;
use crate::tests::adaptive_plan_fixture::{refocus, state};
use crate::tests::adaptive_plan_support::plan_existing;
use ghostr_engine::PostId;

#[test]
fn recent_backward_navigation_preserves_the_bounded_forward_window() {
    let mut state = state();
    refocus(&mut state, 4, 1_000);
    refocus(&mut state, 3, 2_000);
    refocus(&mut state, 2, 3_000);

    let planned = allocated_posts(&plan_existing(state));

    assert!(!planned.contains(&PostId::new("p1")), "{planned:?}");
}
