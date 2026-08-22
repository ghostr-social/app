use crate::evaluation::{EvaluationTracker, PlaybackMetricEvent, PresentationMetricEvent};
use ghostr_engine::playback::PlaybackPhase;
use ghostr_engine::PostId;

#[test]
fn only_explicit_presentation_populates_swipe_to_first_frame_quantiles() {
    let mut metrics = EvaluationTracker::default();
    let post = PostId::new("private-post");
    metrics.focus(post.clone(), 1_000);
    metrics.playback(PlaybackMetricEvent {
        post: post.clone(),
        phase: PlaybackPhase::Playing,
        bitrate_bps: 2_000_000,
        observed_at_ms: 1_100,
    });
    assert_eq!(
        metrics.snapshot().user_visible.swipe_to_first_frame.samples,
        0
    );

    metrics.present(presentation(
        post,
        1_300,
        2_000_000,
        "https://origin.example/a",
    ));
    metrics.focus(PostId::new("second-private-post"), 2_000);
    metrics.present(presentation(
        PostId::new("second-private-post"),
        2_900,
        1_000_000,
        "https://origin.example/b",
    ));
    let snapshot = metrics.snapshot();

    assert_eq!(snapshot.user_visible.swipe_to_first_frame.samples, 2);
    assert_eq!(snapshot.user_visible.swipe_to_first_frame.p50_ms, 300);
    assert_eq!(snapshot.user_visible.swipe_to_first_frame.p95_ms, 900);
    assert_eq!(snapshot.user_visible.swipe_to_first_frame.p99_ms, 900);
    assert_eq!(snapshot.user_visible.startup_failures, 0);
    assert_eq!(snapshot.user_visible.first_frame_quality_bps, 1_500_000);
    assert_eq!(snapshot.semantics.exposure_by_origin.len(), 1);
    assert!(!serde_json::to_string(&snapshot)
        .unwrap()
        .contains("private"));
}

fn presentation(
    post: PostId,
    observed_at_ms: u64,
    bitrate_bps: u64,
    origin: &str,
) -> PresentationMetricEvent {
    PresentationMetricEvent {
        post,
        bitrate_bps,
        origin: origin.into(),
        observed_at_ms,
    }
}
