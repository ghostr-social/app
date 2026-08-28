use crate::chunk::downloader::ResponseAdmission;
use crate::tests::response_driven_promotion_fixture::meta;
use crate::tests::timeline_manager_fixture::TimelineManagerFixture;
use crate::tests::timeline_parser_fixture::GatedTimelineParser;
use core::time::Duration;

pub(super) async fn fixture() -> TimelineManagerFixture {
    let (parser, _started) = GatedTimelineParser::new(None, 1);
    let mut fixture = TimelineManagerFixture::new(parser).await;
    fixture.focus_with(meta());
    assert!(fixture.step().await);
    quiesce(&mut fixture).await;
    fixture
}

pub(super) async fn join(task: tokio::task::JoinHandle<ResponseAdmission>) -> ResponseAdmission {
    tokio::time::timeout(Duration::from_secs(10), task)
        .await
        .expect("one worker step answers admission")
        .expect("admission task remains live")
}

async fn quiesce(fixture: &mut TimelineManagerFixture) {
    tokio::time::timeout(Duration::from_secs(10), async {
        while !fixture.worker.active_actions_for_test().is_empty() {
            assert!(fixture.step().await);
        }
    })
    .await
    .expect("automatic fixture request reaches terminal failure");
}
