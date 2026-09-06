//! HLS playback remains part of the watch timeline for later progressive videos.
mod delivery_fixture;

use delivery_fixture::playback::wait_for_admission;
use delivery_fixture::watch_deadline::playback::progress;
use delivery_fixture::watch_deadline::WatchFixture;

#[tokio::test]
async fn the_current_hls_watch_is_not_skipped_in_progressive_deadlines() {
    let fixture = WatchFixture::start().await.with_hls_current();
    let initial = fixture.focus(0, 1).await;
    assert_eq!(
        fixture.next_deadline(&initial),
        8_000,
        "the current HLS watch precedes the next video"
    );
    fixture.handle().report_playback(progress(1, 1, 7));
    wait_for_admission(fixture.handle()).await;
    let advanced = fixture.focus(0, 2).await;
    assert_eq!(
        fixture.next_deadline(&advanced),
        1_000,
        "native HLS progress conditions its remaining watch"
    );
}
