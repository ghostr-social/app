mod delivery_fixture;
#[path = "delivery_qoe_persistence_test/state.rs"]
mod state;

use core::time::Duration;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::delivery_events::{
    DeliveryPlayback, PlaybackPresentation, PlaybackPresentationIngress,
};
use ghostr_engine::playback::{
    PlaybackObservation, PlaybackObservationSequence, PlaybackPhase, PlaybackSession,
};
use ghostr_engine::PostId;
use state::load_stats;

#[tokio::test]
async fn manager_persists_accepted_playback_qoe_for_the_next_run() {
    let harness = start_harness("delivery-qoe", DeliveryOptions::default());
    let item = sized_item("current", "https://media.example/video.mp4", 16, 1_000);
    harness.handle.update_focus(focus_now(vec![item], 0, 0));
    tokio::task::yield_now().await;
    assert_eq!(
        harness.handle.report_playback_presentation(
            PlaybackPresentation::try_new(session(), 1, 0).expect("valid test fixture")
        ),
        PlaybackPresentationIngress::Accepted,
    );
    harness
        .handle
        .report_playback(sample(1, PlaybackPhase::NetworkStalled));
    wait_for_playback_admission(&harness.handle).await;
    tokio::time::sleep(Duration::from_millis(10)).await;
    harness
        .handle
        .report_playback(sample(2, PlaybackPhase::Playing));

    let path = harness.root.join("qoe_stats.json");
    let stats = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let Some(stats) = load_stats(&path).await else {
                tokio::task::yield_now().await;
                continue;
            };
            if stats.first_frames == 1
                && stats.stall_events == 1
                && stats.stall_total_ms > 0
                && stats.buffer_samples >= 2
            {
                break stats;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("valid test fixture");

    assert!(stats.buffer_samples >= 2);
    assert!(stats.stall_total_ms > 0);
    std::fs::remove_dir_all(&harness.root).ok();
}

fn sample(sequence: u64, phase: PlaybackPhase) -> DeliveryPlayback {
    DeliveryPlayback {
        session: session(),
        sequence: PlaybackObservationSequence::new(sequence),
        observation: PlaybackObservation::try_new(
            Duration::ZERO,
            Duration::from_secs(1),
            1_000,
            phase,
        )
        .expect("valid test fixture"),
    }
}

fn session() -> PlaybackSession {
    PlaybackSession::new(PostId::new("current"), 1)
}

async fn wait_for_playback_admission(handle: &ghostr_delivery::delivery_events::DeliveryHandle) {
    tokio::time::timeout(Duration::from_secs(2), async {
        while handle.playback_admission_snapshot().counters().accepted() == 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("valid test fixture");
}
