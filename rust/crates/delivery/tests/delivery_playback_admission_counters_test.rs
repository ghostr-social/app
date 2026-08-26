mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::delivery_events::DeliveryPlayback;
use ghostr_delivery::playback_admission::{PlaybackAdmissionCounters, PlaybackRejection};
use ghostr_engine::playback::{
    PlaybackObservation, PlaybackObservationSequence, PlaybackPhase, PlaybackSession,
};
use ghostr_engine::PostId;

#[tokio::test]
async fn manager_counts_each_typed_playback_admission_outcome() {
    let harness = start_harness("ghostr-playback-counters", DeliveryOptions::default());
    let item = sized_item("current", "https://media.example/video.mp4", 16, 1_000);
    harness.handle.update_focus(focus_now(vec![item], 0, 0));
    wait_for_count(&harness, 0).await;

    report_and_wait(&harness, update("current", 2, 1), 1).await;
    report_and_wait(&harness, update("other", 3, 1), 2).await;
    report_and_wait(&harness, update("current", 1, 2), 3).await;
    report_and_wait(&harness, update("current", 2, 1), 4).await;

    let snapshot = harness.handle.playback_admission_snapshot();
    let counters = snapshot.counters();
    assert_eq!(counters.accepted(), 1);
    assert_eq!(counters.rejected(PlaybackRejection::InactiveDelivery), 1);
    assert_eq!(counters.rejected(PlaybackRejection::StaleSession), 1);
    assert_eq!(counters.rejected(PlaybackRejection::StaleSequence), 1);
    assert_eq!(snapshot.last_accepted(), Some(&PostId::new("current")));
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}

async fn report_and_wait(
    harness: &delivery_fixture::DeliveryHarness,
    playback: DeliveryPlayback,
    total: u64,
) {
    harness.handle.report_playback(playback);
    wait_for_count(harness, total).await;
}

async fn wait_for_count(harness: &delivery_fixture::DeliveryHarness, total: u64) {
    tokio::time::timeout(Duration::from_secs(1), async {
        while counter_total(harness.handle.playback_admission_snapshot().counters()) < total {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("playback admission counter");
}

fn counter_total(counters: PlaybackAdmissionCounters) -> u64 {
    counters.accepted()
        + counters.rejected(PlaybackRejection::InactiveDelivery)
        + counters.rejected(PlaybackRejection::StaleSession)
        + counters.rejected(PlaybackRejection::StaleSequence)
}

fn update(post: &str, generation: u64, sequence: u64) -> DeliveryPlayback {
    DeliveryPlayback {
        session: PlaybackSession::new(PostId::new(post), generation),
        sequence: PlaybackObservationSequence::new(sequence),
        observation: PlaybackObservation::try_new(
            Duration::ZERO,
            Duration::from_secs(1),
            1_000,
            PlaybackPhase::Playing,
        )
        .expect("valid test fixture"),
    }
}
