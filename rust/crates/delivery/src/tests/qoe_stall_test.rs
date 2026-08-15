use crate::delivery_events::FocusTransition;
use crate::qoe::QoeTracker;
use ghostr_engine::playback::PlaybackPhase;
use ghostr_engine::PostId;

#[test]
fn records_each_stall_and_its_closed_duration_once() {
    let post = PostId::new("clip");
    let mut tracker = QoeTracker::default();
    tracker.focus(Some(post.clone()), FocusTransition::UserNavigation, None, 0);
    tracker.observe(&post, PlaybackPhase::Playing, 2_000, 100);

    tracker.observe(&post, PlaybackPhase::NetworkStalled, 0, 250);
    tracker.observe(&post, PlaybackPhase::NetworkStalled, 0, 300);
    tracker.observe(&post, PlaybackPhase::Playing, 1_000, 450);

    assert_eq!(tracker.stats().stall_events, 1);
    assert_eq!(tracker.stats().stall_total_ms, 200);
}
