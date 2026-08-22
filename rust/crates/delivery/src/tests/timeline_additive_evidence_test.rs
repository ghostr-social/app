use crate::tests::media_timeline_fixture::classic_moov;
use crate::tests::timeline_manager_fixture::TimelineManagerFixture;
use crate::tests::timeline_parser_fixture::GatedTimelineParser;
use ghostr_engine::media_timeline::{parse_mp4_segments, MediaSegment};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn additive_bytes_do_not_withdraw_an_installed_timeline_during_reparse() {
    let moov = classic_moov(100, 10);
    let ready = parse_mp4_segments(&[MediaSegment::new(0, &moov)]).unwrap();
    let (parser, mut started) = GatedTimelineParser::new(Some(ready.clone()), 3);
    let _release = ReleaseAll(parser.clone(), 3);
    let mut fixture = TimelineManagerFixture::new(parser.clone()).await;

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
        .unwrap();
    fixture.handle.storage_changed();
    step(&mut fixture).await;
    assert_eq!(recv(&mut started).await, 1);
    let first_growth = fixture.timeline();

    fixture
        .store
        .write_range(fixture.post.as_str(), 520, b"more")
        .await
        .unwrap();
    fixture.handle.storage_changed();
    step(&mut fixture).await;
    assert_eq!(recv(&mut started).await, 2);
    let second_growth = fixture.timeline();

    parser.release(1);
    step(&mut fixture).await;
    parser.release(2);
    step(&mut fixture).await;
    let after_incomplete = fixture.timeline();
    tokio::fs::remove_dir_all(fixture.root).await.unwrap();
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
    assert!(
        tokio::time::timeout(Duration::from_secs(1), fixture.worker.step())
            .await
            .unwrap()
    );
}

async fn await_timeline(fixture: &mut TimelineManagerFixture) {
    tokio::time::timeout(Duration::from_secs(1), async {
        while fixture.timeline().is_none() {
            assert!(fixture.worker.step().await);
        }
    })
    .await
    .unwrap();
}

async fn recv(receiver: &mut tokio::sync::mpsc::UnboundedReceiver<usize>) -> usize {
    tokio::time::timeout(Duration::from_secs(1), receiver.recv())
        .await
        .unwrap()
        .unwrap()
}
