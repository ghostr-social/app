mod delivery_fixture;
mod fast_start_mp4_fixture;
mod focus_wait_fixture;
mod transform_delivery_fixture;
mod transform_wait_fixture;

use delivery_fixture::evidence::DeliveryEvidence as _;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::{start_harness_with_store, temp_directory};
use fast_start_mp4_fixture::{tail_indexed_mp4, top_level_boxes};
use focus_wait_fixture::wait_for_focus;
use ghostr_delivery::transform::FastStartRemuxBackend;
use ghostr_engine::adaptive::{DecisionOutcome, RecordedWarpCommand};
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use tokio::sync::Mutex;
use transform_delivery_fixture::{report_unsupported, seed_input};
use transform_wait_fixture::wait_for_transform;

#[tokio::test]
async fn exact_tail_mp4_failure_reaches_the_production_fast_start_transform() {
    let bytes = tail_indexed_mp4();
    let root = temp_directory("warp-fast-start-reachability");
    let store = Arc::new(PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    ));
    let item = sized_item(
        "post",
        "https://origin.example/video.mp4",
        bytes.len() as u64,
        1_000,
    );
    let input = seed_input(&store, &item, &bytes).await;
    let options = DeliveryOptions {
        transform: Some(Arc::new(FastStartRemuxBackend::production())),
        ..DeliveryOptions::default()
    };
    let harness = start_harness_with_store(std::sync::Arc::clone(&store), root, options);
    harness.handle.update_focus(focus_now(vec![item], 0, 0));
    wait_for_focus(&harness.cache).await;
    report_unsupported(&harness.handle, &store, input.clone()).await;

    let transformed = wait_for_transform(&store, &input, &harness.handle).await;
    let total = transformed.total_len().expect("derived total");
    let output = store
        .read_range("post", 0..total)
        .await
        .expect("valid test fixture")
        .expect("valid test fixture");
    assert_eq!(top_level_boxes(&output), [*b"ftyp", *b"moov", *b"mdat"]);
    assert!(harness.handle.decision_history().records.iter().any(|record| {
        matches!(record.eventual_outcome, DecisionOutcome::Succeeded { bytes, .. } if bytes == total)
            && matches!(
                record.warp_decision.as_ref().and_then(|item| item.selected.as_ref()),
                Some(selected) if matches!(selected.command, RecordedWarpCommand::Transform { .. })
            )
    }));
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}
