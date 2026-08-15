mod delivery_fixture;

use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::delivery_events::DeliveryPlayback;
use ghostr_delivery::qoe::load_qoe_stats;
use ghostr_engine::playback::{
    PlaybackObservation, PlaybackObservationSequence, PlaybackPhase, PlaybackSession,
};
use ghostr_engine::PostId;
use std::time::Duration;

#[tokio::test]
async fn manager_persists_accepted_playback_qoe_for_the_next_run() {
    let harness = start_harness("delivery-qoe", DeliveryOptions::default());
    let item = sized_item("current", "https://media.example/video.mp4", 16, 1_000);
    harness.handle.update_focus(focus_now(vec![item], 0, 0));
    tokio::task::yield_now().await;
    harness
        .handle
        .report_playback(sample(1, PlaybackPhase::Playing));
    harness
        .handle
        .report_playback(sample(2, PlaybackPhase::NetworkStalled));
    tokio::time::sleep(Duration::from_millis(10)).await;
    harness
        .handle
        .report_playback(sample(3, PlaybackPhase::Playing));

    let path = harness.root.join("qoe_stats.json");
    let stats = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let stats = load_qoe_stats(&path).await;
            if stats.first_frames == 1 && stats.stall_events == 1 {
                break stats;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();

    assert!(stats.buffer_samples >= 2);
    assert!(stats.stall_total_ms > 0);
    std::fs::remove_dir_all(&harness.root).ok();
}

fn sample(sequence: u64, phase: PlaybackPhase) -> DeliveryPlayback {
    DeliveryPlayback {
        session: PlaybackSession::new(PostId::new("current"), 1),
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
