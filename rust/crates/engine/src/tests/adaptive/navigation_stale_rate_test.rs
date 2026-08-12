use crate::adaptive::{NavigationDirection, NavigationHistory};

#[test]
fn navigation_rate_forgets_transitions_outside_the_recent_window() {
    let mut history = NavigationHistory::default();
    history.record(NavigationDirection::Forward, 1_000);
    history.record(NavigationDirection::Backward, 2_000);
    history.record(NavigationDirection::Forward, 12_001);

    let stale = history.snapshot(12_001);
    assert_eq!(stale.forward_swipes_per_minute, 6);
    assert_eq!(stale.backward_swipes_per_minute, 0);
}
