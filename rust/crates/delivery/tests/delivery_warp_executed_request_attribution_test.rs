//! A cold speculative large-range intent is attributed to the exact
//! sparse request that origin admission authorizes and the worker executes.

#[path = "delivery_warp_executed_request_attribution_test/assertions.rs"]
mod assertions;
mod delivery_fixture;

use assertions::{assert_executed_cap, assert_selected_intent};
use delivery_fixture::items::{focus_now, seed_range, sized_item};
use delivery_fixture::media::{hit_log, hits, serve_recording};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_engine::adaptive::DecisionOutcome;

const TOTAL: u64 = 128 * 1024;
const ADMITTED: u64 = 64 * 1024;

#[tokio::test]
async fn cold_speculative_intent_records_the_executed_sparse_cap() {
    let log = hit_log();
    let origin = serve_recording("speculative", vec![7; TOTAL as usize], log.clone()).await;
    let harness = start_harness("warp-executed-attribution", options());
    let current = sized_item("current", &origin, 16, 1_000);
    let adjacent = sized_item("adjacent", &origin, 16, 1_000);
    seed_complete(&harness.store, &current).await;
    seed_complete(&harness.store, &adjacent).await;

    harness.handle.update_focus(focus_now(
        vec![
            current,
            adjacent,
            sized_item("speculative", &origin, TOTAL, 1_000),
        ],
        0,
        0,
    ));
    let record = delivery_fixture::decision::wait_for_terminal_transfer(&harness.handle).await;
    assert_selected_intent(&record);
    assert_executed_cap(&record);
    assert!(matches!(
        record.eventual_outcome,
        DecisionOutcome::Succeeded {
            bytes: ADMITTED,
            ..
        }
    ));
    assert!(hits(&log)
        .iter()
        .any(|hit| hit == "speculative:GET:0-65535"));
    std::fs::remove_dir_all(&harness.root).ok();
}

async fn seed_complete(
    store: &ghostr_partial_store::partial_range_store::PartialRangeStore,
    item: &ghostr_delivery::delivery_events::FocusItem,
) {
    seed_range(store, item, 0, &[1; 16]).await;
    store.finalize(item.post.as_str(), None).await.unwrap();
}

fn options() -> DeliveryOptions {
    let mut options = DeliveryOptions::default();
    options.params.chunk_bytes = TOTAL;
    options
}
