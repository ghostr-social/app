//! Replanning uses remaining watch time instead of another complete viewing period.
mod delivery_fixture;

use delivery_fixture::watch_deadline::WatchFixture;

#[tokio::test]
async fn reported_watch_progress_brings_the_next_video_deadline_forward() {
    let fixture = WatchFixture::start().await;
    let initial = fixture.focus(0, 1).await;
    assert_eq!(fixture.next_deadline(&initial), 8_000, "cold watch horizon");
    let advanced = fixture.focus(7_000, 2).await;
    assert_eq!(
        fixture.next_deadline(&advanced),
        1_000,
        "remaining watch horizon"
    );
}
