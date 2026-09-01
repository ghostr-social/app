mod delivery_fixture;
mod focus_wait_fixture;
mod transform_delivery_fixture;
mod transform_fixture;
mod transform_wait_fixture;

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
use transform_fixture::{FixtureRemux, INPUT, OUTPUT};
use transform_wait_fixture::wait_for_transform;

#[tokio::test]
async fn selected_transform_publishes_exact_derived_representation() {
    let root = temp_directory("warp-transform");
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
    let options = DeliveryOptions {
        transform: Some(Arc::new(FixtureRemux)),
        ..DeliveryOptions::default()
    };
    let harness = start_harness_with_store(std::sync::Arc::clone(&store), root, options);
    harness.handle.update_focus(focus_now(vec![item], 0, 0));
    wait_for_focus(&harness.cache).await;
    report_unsupported(&harness.handle, &store, input.clone()).await;

    let transformed = wait_for_transform(&store, &input, &harness.handle).await;
    assert!(transformed
        .binding()
        .expect("valid test fixture")
        .derives_from(&input));
    assert_eq!(
        store
            .read_range("post", 0..OUTPUT.len() as u64)
            .await
            .expect("valid test fixture"),
        Some(OUTPUT.to_vec())
    );
    let record = harness
        .handle
        .decision_history()
        .records
        .into_iter()
        .find(|record| {
            matches!(record.eventual_outcome, DecisionOutcome::Succeeded { bytes, elapsed_ms }
                if bytes == OUTPUT.len() as u64 && elapsed_ms > 0)
        })
        .expect("timed terminal Transform decision");
    assert_eq!(record.schema_version, 4);
    let actual = record.actual_resources.expect("actual Transform resources");
    assert_eq!(actual.network_bytes, 0);
    assert_eq!(actual.storage_bytes, OUTPUT.len() as u64);
    assert!(actual.cpu_ms <= 5);
    assert_eq!(actual.requests, 0);
    assert!(matches!(
        record
            .warp_decision
            .expect("valid test fixture")
            .selected
            .expect("valid test fixture")
            .command,
        RecordedWarpCommand::Transform { .. }
    ));
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}
