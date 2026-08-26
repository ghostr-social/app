use crate::tests::media_timeline_fixture::classic_moov;
use crate::tests::timeline_manager_fixture::TimelineManagerFixture;
use crate::tests::timeline_parser_fixture::GatedTimelineParser;
use ghostr_engine::media_timeline::{parse_mp4_segments, MediaSegment};
use core::time::Duration;

#[tokio::test]
async fn control_wakes_bypass_a_blocked_parse_and_stale_geometry_never_installs() {
    let moov = classic_moov(100, 10);
    let ready = parse_mp4_segments(&[MediaSegment::new(0, &moov)]).expect("valid test fixture");
    let (parser, mut started) = GatedTimelineParser::new(Some(ready), 2);
    let mut fixture = TimelineManagerFixture::new(std::sync::Arc::<GatedTimelineParser>::clone(&parser)).await;

    fixture.focus();
    assert!(fixture.step().await);
    assert_eq!(started.recv().await, Some(0));
    fixture
        .store
        .write_range(fixture.post.as_str(), 0, b"replacement")
        .await
        .expect("valid test fixture");
    fixture.handle.storage_changed();

    let responsive = tokio::time::timeout(Duration::from_secs(1), fixture.step()).await;
    assert!(responsive.expect("valid test fixture"));
    assert_eq!(started.recv().await, Some(1));

    parser.release(1);
    assert!(fixture.step().await);
    assert!(fixture.timeline().is_none());
    parser.release(0);
    assert!(fixture.step().await);
    assert!(fixture.timeline().is_none());

    tokio::fs::remove_dir_all(fixture.root).await.expect("valid test fixture");
}
