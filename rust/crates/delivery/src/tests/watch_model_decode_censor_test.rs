use crate::qoe::axiom_test_support::WatchOutcome;
use crate::delivery_events::FocusTransition;
use crate::qoe::WatchLearner;
use crate::tests::watch_model_fixture::{focus, playback};
use ghostr_engine::playback::PlaybackPhase;
use ghostr_engine::watch_model::{WatchCensor, WatchSampleKind};

#[test]
fn decode_failure_is_censored_without_training_abandonment() {
    let mut learner = WatchLearner::default();
    learner.focus(&focus(1, 0, FocusTransition::UserNavigation), 10);
    let revision = learner.model().revision();

    learner.playback(&playback("b", 1, 9_500, PlaybackPhase::Failed), 20);

    assert_eq!(
        learner.last_outcome(),
        Some(WatchOutcome::sample(
            9_500,
            WatchSampleKind::Censored(WatchCensor::DecodeFailure),
        ))
    );
    assert_eq!(learner.model().revision(), revision);
}
