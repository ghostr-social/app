//! Current playback speed determines the wall-clock deadline for the next video.
mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::playback::wait_for_admissions;
use delivery_fixture::watch_deadline::playback::progress;
use delivery_fixture::watch_deadline::WatchFixture;
use ghostr_engine::playback::{PlaybackObservation, PlaybackPhase};

#[tokio::test]
async fn changing_playback_rate_retimes_the_remaining_watch_deadline() {
    let fixture = WatchFixture::start().await;
    fixture.focus(0, 1).await;
    fixture.handle().report_playback(progress(1, 1, 7));
    wait_for_admissions(fixture.handle(), 1).await;
    let normal = fixture.focus(0, 2).await;
    assert_eq!(
        fixture.next_deadline(&normal),
        1_000,
        "one second remains at normal speed"
    );
    let mut fast = progress(1, 2, 7);
    fast.observation = PlaybackObservation::try_new(
        Duration::from_secs(7),
        Duration::from_secs(8),
        2_000,
        PlaybackPhase::Playing,
    )
    .expect("double-speed playback");
    fixture.handle().report_playback(fast);
    wait_for_admissions(fixture.handle(), 2).await;
    let accelerated = fixture.focus(0, 3).await;
    assert_eq!(
        fixture.next_deadline(&accelerated),
        500,
        "the next video can be needed twice as soon"
    );
}
