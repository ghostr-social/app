use crate::delivery_events::FocusTransition;
use crate::qoe::QoeTracker;
use ghostr_engine::playback::PlaybackPhase;
use ghostr_engine::PostId;

#[test]
fn inactive_before_end_is_abandonment_but_completion_is_not() {
    let first = PostId::new("first");
    let second = PostId::new("second");
    let mut tracker = QoeTracker::default();
    tracker.focus(
        Some(first.clone()),
        FocusTransition::UserNavigation,
        None,
        0,
    );
    tracker.observe(&first, PlaybackPhase::Starting, 0, 10);
    tracker.observe(&first, PlaybackPhase::Inactive, 0, 20);
    tracker.focus(
        Some(second.clone()),
        FocusTransition::UserNavigation,
        None,
        30,
    );
    tracker.observe(&second, PlaybackPhase::Playing, 1_000, 40);
    tracker.present(&second, 40);
    tracker.observe(&second, PlaybackPhase::Ended, 0, 50);
    tracker.observe(&second, PlaybackPhase::Inactive, 0, 60);

    assert_eq!(tracker.stats().abandonments, 1);
    assert_eq!(tracker.stats().startup_failures, 1);
    assert_eq!(tracker.stats().completions, 1);
}
