//! Playback evidence builders shared by delivery integration scenarios.

use core::time::Duration;
use ghostr_delivery::delivery_events::DeliveryPlayback;
use ghostr_engine::playback::{
    PlaybackObservation, PlaybackObservationSequence, PlaybackPhase, PlaybackSession,
};
use ghostr_engine::PostId;

pub async fn wait_for_admission(handle: &ghostr_delivery::delivery_events::DeliveryHandle) {
    wait_for_admissions(handle, 1).await;
}

pub async fn wait_for_admissions(
    handle: &ghostr_delivery::delivery_events::DeliveryHandle,
    expected: u64,
) {
    tokio::time::timeout(Duration::from_secs(2), async {
        while handle.playback_admission_snapshot().counters().accepted() < expected {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("accepted playback fixture");
}

pub fn playing(post: &str, buffer: Duration) -> DeliveryPlayback {
    playing_at(post, buffer, 1)
}

pub fn playing_at(post: &str, buffer: Duration, sequence: u64) -> DeliveryPlayback {
    DeliveryPlayback {
        session: PlaybackSession::new(PostId::new(post), 1),
        sequence: PlaybackObservationSequence::new(sequence),
        observation: PlaybackObservation::try_new(
            Duration::ZERO,
            buffer,
            1_000,
            PlaybackPhase::Playing,
        )
        .expect("valid test fixture"),
    }
}
