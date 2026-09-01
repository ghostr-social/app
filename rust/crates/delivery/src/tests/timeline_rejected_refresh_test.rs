use crate::tests::media_timeline_fixture::classic_moov;
use crate::tests::timeline_manager_fixture::TimelineManagerFixture;
use crate::tests::timeline_parser_fixture::GatedTimelineParser;
use core::time::Duration;
use ghostr_engine::media_timeline::{parse_mp4_segments, MediaSegment};
use std::sync::Arc;

#[tokio::test]
async fn contradictory_additive_evidence_withdraws_the_preserved_timeline() {
    let moov = classic_moov(100, 10);
    let ready = parse_mp4_segments(&[MediaSegment::new(0, &moov)]).expect("valid test fixture");
    let (parser, mut started) = GatedTimelineParser::rejecting_refresh(ready);
    let _release = ReleaseAll(std::sync::Arc::clone(&parser));
    let mut fixture =
        TimelineManagerFixture::new(std::sync::Arc::<GatedTimelineParser>::clone(&parser)).await;

    fixture.focus();
    step(&mut fixture).await;
    assert_eq!(recv(&mut started).await, 0);
    parser.release(0);
    await_timeline(&mut fixture).await;
    fixture
        .store
        .write_range(fixture.post.as_str(), 512, b"contradiction")
        .await
        .expect("valid test fixture");
    fixture.handle.storage_changed();
    step(&mut fixture).await;
    assert_eq!(recv(&mut started).await, 1);
    let preserved = fixture.timeline();

    parser.release(1);
    await_no_timeline(&mut fixture).await;
    tokio::fs::remove_dir_all(fixture.root)
        .await
        .expect("valid test fixture");
    assert!(preserved.is_some());
}

struct ReleaseAll(Arc<GatedTimelineParser>);

impl Drop for ReleaseAll {
    fn drop(&mut self) {
        self.0.release(0);
        self.0.release(1);
    }
}

async fn step(fixture: &mut TimelineManagerFixture) {
    assert!(tokio::time::timeout(Duration::from_secs(1), fixture.step())
        .await
        .expect("valid test fixture"));
}

async fn await_timeline(fixture: &mut TimelineManagerFixture) {
    tokio::time::timeout(Duration::from_secs(1), async {
        while fixture.timeline().is_none() {
            assert!(fixture.step().await);
        }
    })
    .await
    .expect("valid test fixture");
}

async fn await_no_timeline(fixture: &mut TimelineManagerFixture) {
    tokio::time::timeout(Duration::from_secs(1), async {
        while fixture.timeline().is_some() {
            assert!(fixture.step().await);
        }
    })
    .await
    .expect("valid test fixture");
}

async fn recv(receiver: &mut tokio::sync::mpsc::UnboundedReceiver<usize>) -> usize {
    tokio::time::timeout(Duration::from_secs(1), receiver.recv())
        .await
        .expect("valid test fixture")
        .expect("valid test fixture")
}
