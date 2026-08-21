mod delivery_fixture;
mod focus_wait_fixture;
mod transform_delivery_fixture;
mod transform_fixture;
mod transform_wait_fixture;

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
    let harness = start_harness_with_store(store.clone(), root, options);
    harness.handle.update_focus(focus_now(vec![item], 0, 0));
    wait_for_focus(&harness.cache).await;
    report_unsupported(&harness.handle, &store, input.clone()).await;

    let transformed = wait_for_transform(&store, &input, &harness.handle).await;
    assert!(transformed.binding().unwrap().derives_from(&input));
    assert_eq!(
        store
            .read_range("post", 0..OUTPUT.len() as u64)
            .await
            .unwrap(),
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
    assert_eq!(record.schema_version, 2);
    assert!(matches!(
        record.warp_decision.unwrap().selected.unwrap().command,
        RecordedWarpCommand::Transform { .. }
    ));
    harness.handle.clear().await.unwrap();
    std::fs::remove_dir_all(&harness.root).ok();
}
