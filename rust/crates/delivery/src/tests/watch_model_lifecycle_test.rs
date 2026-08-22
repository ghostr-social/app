use crate::delivery_events::FocusTransition;
use crate::qoe::{WatchLearner, WatchOutcome};
use crate::tests::watch_model_fixture::{focus, playback};
use ghostr_engine::playback::PlaybackPhase;
use ghostr_engine::watch_model::{WatchNavigation, WatchSampleKind};

#[test]
fn accepted_lifecycle_uses_exact_position_for_abandonment_and_completion() {
    let mut learner = WatchLearner::default();
    learner.focus(&focus(0, 0, FocusTransition::UserNavigation), 10);
    learner.playback(&playback("a", 1, 1_250, PlaybackPhase::Playing), 20);
    learner.focus(&focus(1, 2_000, FocusTransition::UserNavigation), 30);
    assert_eq!(
        learner.last_outcome(),
        Some(WatchOutcome::sample(2_000, WatchSampleKind::Abandoned,))
    );
    assert_eq!(learner.last_navigation(), Some(WatchNavigation::Forward));

    learner.playback(&playback("b", 1, 9_000, PlaybackPhase::Ended), 40);
    assert_eq!(
        learner.last_outcome(),
        Some(WatchOutcome::sample(9_000, WatchSampleKind::Completed,))
    );
    let revision = learner.model().revision();
    learner.playback(&playback("b", 2, 9_500, PlaybackPhase::Failed), 50);
    assert_eq!(
        learner.last_outcome(),
        Some(WatchOutcome::sample(9_000, WatchSampleKind::Completed,))
    );
    assert_eq!(learner.model().revision(), revision);
}
