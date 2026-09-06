//! Native progress survives zero-watch roster refreshes.
mod delivery_fixture;

use delivery_fixture::playback::wait_for_admission;
use delivery_fixture::watch_deadline::playback::progress;
use delivery_fixture::watch_deadline::WatchFixture;

#[tokio::test]
async fn admitted_native_progress_conditions_the_next_deadline() {
    let fixture = WatchFixture::start().await;
    fixture.focus(0, 1).await;
    fixture.handle().report_playback(progress(1, 1, 7));
    wait_for_admission(fixture.handle()).await;
    let advanced = fixture.focus(0, 2).await;
    assert_eq!(
        fixture.next_deadline(&advanced),
        1_000,
        "native progress reaches planner"
    );
}
