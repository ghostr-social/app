
use crate::manager::timeline::{TimelineCoordinator, TimelineEvidence, TimelineJobOutcome, TimelineSchedule, TimelineTerminal};
use crate::tests::demand_lease_fixture::{binding, catalog};
use crate::tests::media_timeline_fixture::classic_moov;
use crate::tests::support::temp_directory;
use ghostr_engine::media_timeline::{parse_mp4_segments, MediaSegment};
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use core::time::Duration;
use tokio::sync::Mutex;

#[tokio::test]
async fn overlapping_head_and_tail_metadata_is_read_and_parsed_once() {
    let root = temp_directory("timeline-overlap");
    let store = Arc::new(PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    ));
    let binding = binding(&catalog(&["post"]), "post");
    let total: u64 = 6 * 1024 * 1024;
    let moov_start: u64 = 3 * 1024 * 1024;
    let moov = classic_moov((moov_start + 1_000) as u32, 10);
    let mut body = top_level_body(total, moov_start);
    body[moov_start as usize..moov_start as usize + moov.len()].copy_from_slice(&moov);
    store.bind_representation(binding.clone()).await.expect("valid test fixture");
    store.set_total_len("post", total).await.expect("valid test fixture");
    store.write_range("post", 0, &body).await.expect("valid test fixture");
    let snapshot = store.media_snapshot("post").await.expect("valid test fixture");
    let evidence = TimelineEvidence::from_snapshot(&binding, &snapshot).expect("valid test fixture");
    let expected = parse_mp4_segments(&[MediaSegment::new(0, &body)]).expect("valid test fixture");
    let mut coordinator = TimelineCoordinator::new(store);

    let post = PostId::new("post");
    assert_eq!(
        coordinator.schedule(post.clone(), evidence.clone()),
        TimelineSchedule::Started
    );
    coordinator.dispatch(core::slice::from_ref(&post));
    let result = tokio::time::timeout(Duration::from_secs(2), coordinator.recv())
        .await
        .expect("valid test fixture")
        .expect("valid test fixture");
    let outcome = coordinator.validate(result, Some(&evidence));

    let Some(TimelineJobOutcome::Terminal(TimelineTerminal::Ready(actual))) = outcome else {
        panic!("overlapping metadata did not produce a ready timeline");
    };
    assert_eq!(*actual, expected);
    tokio::fs::remove_dir_all(root).await.expect("valid test fixture");
}

fn top_level_body(total: u64, movie_start: u64) -> Vec<u8> {
    let mut body = vec![0; total as usize];
    body[..4].copy_from_slice(&16_u32.to_be_bytes());
    body[4..8].copy_from_slice(b"ftyp");
    body[8..12].copy_from_slice(b"isom");
    let media_size = u32::try_from(movie_start - 16).expect("valid test fixture");
    body[16..20].copy_from_slice(&media_size.to_be_bytes());
    body[20..24].copy_from_slice(b"mdat");
    body
}
