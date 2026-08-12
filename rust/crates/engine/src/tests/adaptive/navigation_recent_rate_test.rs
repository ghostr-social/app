use crate::adaptive::{NavigationDirection, NavigationHistory};

#[test]
fn recent_directional_transitions_produce_a_per_minute_rate() {
    let mut history = NavigationHistory::default();
    history.record(NavigationDirection::Forward, 1_000);
    history.record(NavigationDirection::Forward, 2_000);
    history.record(NavigationDirection::Backward, 3_000);

    let recent = history.snapshot(3_000);
    assert_eq!(recent.forward_swipes_per_minute, 12);
    assert_eq!(recent.backward_swipes_per_minute, 6);
}
