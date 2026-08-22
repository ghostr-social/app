use crate::evaluation::{EvaluationTracker, PresentationMetricEvent, TransferMetricEvent};
use ghostr_engine::PostId;

#[test]
fn full_download_is_unused_until_explicit_presentation() {
    let mut tracker = EvaluationTracker::default();
    let post = PostId::new("private-post");
    tracker.focus(post.clone(), 1_000);
    tracker.transfer(TransferMetricEvent {
        post: Some(post.clone()),
        completable_probe_bytes: 65_536,
        full_download_started: true,
        ..TransferMetricEvent::default()
    });

    let before = tracker.snapshot();
    assert_eq!(before.efficiency.full_downloads_never_useful, 1);
    assert_eq!(before.efficiency.completable_probe_bytes, 65_536);

    tracker.present(PresentationMetricEvent {
        post,
        bitrate_bps: 1_000_000,
        origin: "private-origin".into(),
        observed_at_ms: 1_100,
    });
    assert_eq!(tracker.snapshot().efficiency.full_downloads_never_useful, 0);
}
