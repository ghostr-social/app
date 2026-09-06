//! Rejected native observations cannot change watch conditioning.
mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::playback::wait_for_admission;
use delivery_fixture::watch_deadline::playback::progress;
use delivery_fixture::watch_deadline::WatchFixture;
use ghostr_delivery::playback_admission::PlaybackRejection;

#[tokio::test]
async fn a_stale_native_sequence_cannot_advance_the_conditional_deadline() {
    let fixture = WatchFixture::start().await;
    fixture.focus(0, 1).await;
    fixture.handle().report_playback(progress(1, 2, 2));
    wait_for_admission(fixture.handle()).await;
    fixture.handle().report_playback(progress(1, 1, 7));
    wait_for_rejection(&fixture).await;
    let unchanged = fixture.focus(0, 2).await;
    assert_eq!(
        fixture.next_deadline(&unchanged),
        6_000,
        "only admitted progress counts"
    );
}

async fn wait_for_rejection(fixture: &WatchFixture) {
    tokio::time::timeout(Duration::from_secs(2), async {
        while fixture
            .handle()
            .playback_admission_snapshot()
            .counters()
            .rejected(PlaybackRejection::StaleSequence)
            == 0
        {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("stale progress was rejected");
}
