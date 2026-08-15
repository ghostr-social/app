use crate::delivery_events::FocusTransition;
use crate::qoe::QoeTracker;
use ghostr_engine::playback::PlaybackPhase;
use ghostr_engine::PostId;

#[test]
fn first_playing_sample_learns_startup_and_buffer_quality() {
    let post = PostId::new("clip");
    let mut tracker = QoeTracker::default();
    tracker.focus(
        Some(post.clone()),
        FocusTransition::UserNavigation,
        None,
        100,
    );

    tracker.observe(&post, PlaybackPhase::Playing, 2_000, 350);

    let stats = tracker.stats();
    assert_eq!(stats.first_frames, 1);
    assert_eq!(stats.startup_total_ms, 250);
    assert_eq!(stats.buffer_samples, 1);
    assert_eq!(stats.buffer_ahead_total_ms, 2_000);
    assert_ne!(stats.startup_eta_ms(), QoeTracker::DEFAULT_STARTUP_ETA_MS);
}
