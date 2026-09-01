use crate::delivery_events::DeliveryFocus;
use crate::tests::media_timeline_fixture::classic_moov;
use crate::tests::timeline_manager_fixture::TimelineManagerFixture;
use crate::tests::timeline_parser_fixture::GatedTimelineParser;
use core::time::Duration;
use ghostr_engine::budget::params_for;
use ghostr_engine::media_timeline::{parse_mp4_segments, MediaSegment};
use ghostr_engine::{DataUsageLevel, EngineParams};

#[tokio::test]
async fn queued_focus_is_applied_before_a_simultaneously_ready_timeline_result() {
    let moov = classic_moov(100, 10);
    let ready = parse_mp4_segments(&[MediaSegment::new(0, &moov)]).expect("valid test fixture");
    let (parser, mut started) = GatedTimelineParser::new(Some(ready), 1);
    let mut fixture =
        TimelineManagerFixture::new(std::sync::Arc::<GatedTimelineParser>::clone(&parser)).await;

    fixture.focus();
    assert!(fixture.step().await);
    assert_eq!(started.recv().await, Some(0));
    parser.release(0);
    wait_until_result_is_ready(&mut fixture).await;
    fixture.handle.set_data_usage(DataUsageLevel::Conservative);
    fixture
        .handle
        .update_focus(DeliveryFocus::compatibility(Vec::new(), 0, 0));

    assert!(fixture.step().await);
    assert_eq!(
        fixture.worker.params_for_test(),
        params_for(DataUsageLevel::Conservative, EngineParams::default())
    );
    assert!(fixture.worker.current_post_for_test().is_none());
    assert!(fixture.timeline().is_none());
    tokio::fs::remove_dir_all(fixture.root)
        .await
        .expect("valid test fixture");
}

async fn wait_until_result_is_ready(fixture: &mut TimelineManagerFixture) {
    tokio::time::timeout(Duration::from_secs(1), async {
        while !fixture.worker.timeline_result_ready_for_test() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("valid test fixture");
}
