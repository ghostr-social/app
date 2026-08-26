use crate::manager::timeline::axiom_test_support::load_timeline;

use crate::tests::media_timeline_fixture::classic_moov;
use crate::tests::support::temp_directory;
use ghostr_engine::{ByteRange, PostId};
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn disjoint_tail_metadata_hydrates_without_reading_the_file_middle() {
    let root = temp_directory("delivery-tail-timeline");
    let store = PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    );
    let moov = classic_moov(32, 100);
    let moov_start = 10_000_u64;
    let mut tail = vec![0xa5; 200];
    tail.extend_from_slice(&moov);
    store
        .write_range("post", moov_start - 200, &tail)
        .await
        .expect("valid test fixture");
    let total = moov_start + moov.len() as u64;
    store.set_total_len("post", total).await.expect("valid test fixture");
    let present = [ByteRange::new(
        moov_start - 200,
        moov_start + moov.len() as u64,
    )];

    let timeline = load_timeline(&store, &PostId::new("post"), total, &present)
        .await
        .expect("tail timeline");

    assert!(timeline.fits_within(total));
    tokio::fs::remove_dir_all(root).await.expect("valid test fixture");
}

#[tokio::test]
async fn incomplete_metadata_keeps_the_planner_on_its_safe_fallback() {
    let root = temp_directory("delivery-incomplete-timeline");
    let store = PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    );
    store
        .write_range("post", 0, &[0, 0, 1, 0, b'm', b'o', b'o', b'v'])
        .await
        .expect("valid test fixture");

    let timeline = load_timeline(
        &store,
        &PostId::new("post"),
        20_000,
        &[ByteRange::new(0, 8)],
    )
    .await;

    assert!(timeline.is_none());
    tokio::fs::remove_dir_all(root).await.expect("valid test fixture");
}

#[tokio::test]
async fn sample_offsets_beyond_the_representation_are_not_authorized() {
    let root = temp_directory("delivery-out-of-file-timeline");
    let store = PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    );
    let moov = classic_moov(100_000, 100);
    store.write_range("post", 0, &moov).await.expect("valid test fixture");
    let total = moov.len() as u64;
    store.set_total_len("post", total).await.expect("valid test fixture");

    let timeline = load_timeline(
        &store,
        &PostId::new("post"),
        total,
        &[ByteRange::new(0, total)],
    )
    .await;

    assert!(timeline.is_none());
    tokio::fs::remove_dir_all(root).await.expect("valid test fixture");
}
