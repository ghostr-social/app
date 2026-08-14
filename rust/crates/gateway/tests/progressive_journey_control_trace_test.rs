mod gateway_fixture;

use gateway_fixture::progressive_delivery::ProgressiveDeliveryHarness;
use ghostr_delivery::delivery_events::{DeliveryPlayback, FocusItem};
use ghostr_engine::playback::{
    PlaybackObservation, PlaybackObservationSequence, PlaybackPhase, PlaybackSession,
};
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::time::Duration;

#[tokio::test]
async fn progressive_journey_records_control_and_player_events() {
    let harness = ProgressiveDeliveryHarness::start("ghostr-progressive-control-trace");
    harness.focus(vec![item("delivery-current")], 0);
    harness.observe(playback(PlaybackPhase::Playing, 1));
    harness.first_frame("delivery-current");
    harness.observe(playback(PlaybackPhase::NetworkStalled, 2));
    harness.cancel("delivery-current");

    assert_eq!(
        harness.trace.focuses()[0].posts,
        vec![PostId::new("delivery-current")]
    );
    assert_eq!(harness.trace.observations().len(), 2);
    assert_eq!(
        harness.trace.stalls(),
        vec![PostId::new("delivery-current")]
    );
    assert_eq!(
        harness.trace.first_frames(),
        vec![PostId::new("delivery-current")]
    );
    assert_eq!(
        harness.trace.cancellations(),
        vec![PostId::new("delivery-current")]
    );
}

fn playback(phase: PlaybackPhase, sequence: u64) -> DeliveryPlayback {
    DeliveryPlayback {
        session: PlaybackSession::new(PostId::new("delivery-current"), 1),
        sequence: PlaybackObservationSequence::new(sequence),
        observation: PlaybackObservation::try_new(
            Duration::ZERO,
            Duration::from_secs(1),
            1_000,
            phase,
        )
        .unwrap(),
    }
}

fn item(post: &str) -> FocusItem {
    FocusItem {
        post: PostId::new(post),
        meta: VideoMeta {
            urls: vec!["https://media.example/video.mp4".to_owned()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(285_652),
            duration_ms: Some(6_000),
        },
    }
}
