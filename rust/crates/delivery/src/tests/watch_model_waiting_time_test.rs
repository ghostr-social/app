use crate::delivery_events::FocusTransition;
use crate::qoe::WatchLearner;
use crate::tests::watch_model_fixture::{focus, playback};
use core::time::Duration;
use ghostr_engine::playback::PlaybackPhase;
use ghostr_engine::PostId;

#[test]
fn paused_and_stalled_wall_time_cannot_become_consumed_media() {
    let mut learner = WatchLearner::default();
    learner.focus(&focus(0, 0, FocusTransition::UserNavigation), 0);
    learner.playback(&playback("a", 1, 7_000, PlaybackPhase::Playing), 7_000);
    learner.playback(&playback("a", 2, 7_000, PlaybackPhase::Paused), 60_000);
    assert_eq!(
        learner.current_watch(Some(&PostId::new("a"))),
        Duration::from_secs(7),
        "pause dwell is not watching"
    );
    learner.playback(
        &playback("a", 3, 7_000, PlaybackPhase::NetworkStalled),
        120_000,
    );
    assert_eq!(
        learner.current_watch(Some(&PostId::new("a"))),
        Duration::from_secs(7),
        "waiting for the network is not watching"
    );
}
