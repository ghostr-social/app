use crate::adaptive::{NavigationDirection, NavigationHistory};

#[test]
fn navigation_history_keeps_a_bounded_recent_sample() {
    let mut history = NavigationHistory::default();
    for observed_at_ms in 0..100 {
        history.record(NavigationDirection::Forward, observed_at_ms);
    }

    assert_eq!(history.snapshot(100).forward_swipes_per_minute, 384);
}
