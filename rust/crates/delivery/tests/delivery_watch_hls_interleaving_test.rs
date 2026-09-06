//! Forecasts preserve playlist-based videos between progressive candidates.
mod delivery_fixture;

use delivery_fixture::watch_deadline::WatchFixture;

#[tokio::test]
async fn an_intermediate_hls_video_keeps_its_place_in_the_watch_timeline() {
    let fixture = WatchFixture::start().await.with_hls_middle();
    let initial = fixture.focus(0, 1).await;
    assert_eq!(
        fixture.deadline_at(&initial, 2),
        20_000,
        "the current and intervening HLS watches both precede the third video"
    );
}
