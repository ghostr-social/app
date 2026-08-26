mod blocking_transform_fixture;
mod delivery_fixture;
mod focus_wait_fixture;
mod transform_delivery_fixture;

use blocking_transform_fixture::BlockingRemux;
use core::time::Duration;
use delivery_fixture::decision::wait_for_history;
use delivery_fixture::evidence::DeliveryEvidence as _;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::{start_harness_with_store, temp_directory};
use focus_wait_fixture::wait_for_focus;
use ghostr_engine::adaptive::{DecisionOutcome, RecordedWarpCommand};
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use tokio::sync::Mutex;
use transform_delivery_fixture::{report_unsupported, seed_input};

const INPUT: &[u8] = b"ftyp|mdat:frames|moov:index";

#[tokio::test]
async fn clear_cancels_selected_transform_without_publishing_output() {
    let root = temp_directory("warp-transform-cancel");
    let store = Arc::new(PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    ));
    let item = sized_item(
        "post",
        "https://origin.example/video.mp4",
        INPUT.len() as u64,
        1_000,
    );
    let input = seed_input(&store, &item, INPUT).await;
    let backend = Arc::new(BlockingRemux::new());
    let options = DeliveryOptions {
        transform: Some(Arc::<BlockingRemux>::clone(&backend)),
        ..DeliveryOptions::default()
    };
    let harness = start_harness_with_store(std::sync::Arc::clone(&store), root, options);
    harness.handle.update_focus(focus_now(vec![item], 0, 0));
    wait_for_focus(&harness.cache).await;
    report_unsupported(&harness.handle, &store, input).await;
    backend.wait_until_entered().await;
    tokio::time::sleep(Duration::from_millis(2)).await;

    harness.handle.clear().await.expect("valid test fixture");
    wait_for_history(&harness.handle, |history| {
        history.records.iter().any(cancelled_transform)
    })
    .await;
    let record = harness
        .handle
        .decision_history()
        .records
        .into_iter()
        .find(cancelled_transform)
        .expect("cancelled Transform decision");
    assert_eq!(record.schema_version, 3);
    let actual = record.actual_resources.expect("cancelled CPU evidence");
    assert!(actual.cpu_ms > 0);
    assert_eq!(actual.storage_bytes, 0);
    assert!(store
        .read_range("post", 0..INPUT.len() as u64)
        .await
        .expect("valid test fixture")
        .is_none());
    std::fs::remove_dir_all(&harness.root).ok();
}

fn cancelled_transform(record: &ghostr_engine::adaptive::DecisionRecord) -> bool {
    matches!(
        record.eventual_outcome,
        DecisionOutcome::Cancelled {
            bytes: 0,
            elapsed_ms
        } if elapsed_ms > 0
    ) && matches!(
        record
            .warp_decision
            .as_ref()
            .and_then(|warp| warp.selected.as_ref())
            .map(|action| &action.command),
        Some(RecordedWarpCommand::Transform { .. })
    ) && record.actual_resources.is_some()
}
