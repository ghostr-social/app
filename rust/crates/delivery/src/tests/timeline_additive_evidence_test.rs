use crate::tests::media_timeline_fixture::classic_moov;
use crate::tests::timeline_manager_fixture::TimelineManagerFixture;
use crate::tests::timeline_parser_fixture::GatedTimelineParser;
use core::time::Duration;
use ghostr_engine::media_timeline::{parse_mp4_segments, MediaSegment};
use std::sync::Arc;

const WAIT_LIMIT: Duration = Duration::from_secs(30);

#[tokio::test]
async fn additive_bytes_do_not_withdraw_an_installed_timeline_during_reparse() {
    let moov = classic_moov(100, 10);
    let ready = parse_mp4_segments(&[MediaSegment::new(0, &moov)]).expect("valid test fixture");
    let (parser, mut started) = GatedTimelineParser::new(Some(ready.clone()), 3);
    let _release = ReleaseAll(std::sync::Arc::clone(&parser), 3);
    let mut fixture =
        TimelineManagerFixture::new(std::sync::Arc::<GatedTimelineParser>::clone(&parser)).await;

    fixture.focus();
    step(&mut fixture).await;
    assert_eq!(recv(&mut started).await, 0);
    parser.release(0);
    await_timeline(&mut fixture).await;
    assert_eq!(fixture.timeline(), Some(ready.clone()));

    fixture
        .store
        .write_range(fixture.post.as_str(), 512, b"additive")
        .await
        .expect("valid test fixture");
    fixture.handle.storage_changed();
    step(&mut fixture).await;
    assert_eq!(recv(&mut started).await, 1);
    let first_growth = fixture.timeline();

    fixture
        .store
        .write_range(fixture.post.as_str(), 520, b"more")
        .await
        .expect("valid test fixture");
    fixture.handle.storage_changed();
    step(&mut fixture).await;
    assert_eq!(recv(&mut started).await, 2);
    let second_growth = fixture.timeline();

    parser.release(1);
    step(&mut fixture).await;
    parser.release(2);
    step(&mut fixture).await;
    let after_incomplete = fixture.timeline();
    tokio::fs::remove_dir_all(fixture.root)
        .await
        .expect("valid test fixture");
    assert_eq!(first_growth, Some(ready.clone()));
    assert_eq!(second_growth, Some(ready.clone()));
    assert_eq!(after_incomplete, Some(ready));
}

struct ReleaseAll(Arc<GatedTimelineParser>, usize);

impl Drop for ReleaseAll {
    fn drop(&mut self) {
        for gate in 0..self.1 {
            self.0.release(gate);
        }
    }
}

async fn step(fixture: &mut TimelineManagerFixture) {
    assert!(tokio::time::timeout(WAIT_LIMIT, fixture.step())
        .await
        .expect("valid test fixture"));
}

async fn await_timeline(fixture: &mut TimelineManagerFixture) {
    tokio::time::timeout(WAIT_LIMIT, async {
        while fixture.timeline().is_none() {
            assert!(fixture.step().await);
        }
    })
    .await
    .expect("valid test fixture");
}

async fn recv(receiver: &mut tokio::sync::mpsc::UnboundedReceiver<usize>) -> usize {
    tokio::time::timeout(WAIT_LIMIT, receiver.recv())
        .await
        .expect("valid test fixture")
        .expect("valid test fixture")
}
