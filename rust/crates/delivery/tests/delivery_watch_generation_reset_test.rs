//! A replacement player session does not inherit another session's progress.
mod delivery_fixture;

use delivery_fixture::playback::wait_for_admissions;
use delivery_fixture::watch_deadline::playback::progress;
use delivery_fixture::watch_deadline::WatchFixture;

#[tokio::test]
async fn a_new_native_generation_resets_the_conditional_deadline() {
    let fixture = WatchFixture::start().await;
    fixture.focus(0, 1).await;
    fixture.handle().report_playback(progress(1, 1, 7));
    wait_for_admissions(fixture.handle(), 1).await;
    let advanced = fixture.focus(0, 2).await;
    assert_eq!(
        fixture.next_deadline(&advanced),
        1_000,
        "previous session progress"
    );
    fixture.handle().report_playback(progress(2, 1, 0));
    wait_for_admissions(fixture.handle(), 2).await;
    let reset = fixture.focus(0, 3).await;
    assert_eq!(
        fixture.next_deadline(&reset),
        8_000,
        "replacement session starts afresh"
    );
}
