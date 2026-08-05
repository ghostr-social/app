//! A stale A1 completion cannot evict the newer A2 transfer grant.

mod range_fixture;
mod support;

use range_fixture::stall::serve_stalling;
use rust_lib_ghostr::engine::{DataUsageLevel, EngineParams};
use std::time::Duration;
use support::delivery::start_harness;
use support::delivery_aba_origin::serve;
use support::delivery_items::{focus_now, sized_item};
use support::delivery_options::{base_params, DeliveryOptions};
use support::delivery_wait::wait_for_file;

#[tokio::test]
async fn delivery_manager_ignores_stale_completion_after_focus_returns_to_a() {
    let bytes = b"abcdefgh".to_vec();
    let (a_url, origin) = serve(bytes.clone()).await;
    let b_url = serve_stalling(Vec::new(), 8).await;
    let harness = start_harness("ghostr-delivery-aba", serial_options());
    let a = sized_item("aa11", &a_url, 8, 1_000);

    harness
        .handle
        .update_focus(focus_now(vec![a.clone()], 0, 0));
    origin.wait_for_hits(1).await;
    harness
        .handle
        .update_focus(focus_now(vec![sized_item("bb22", &b_url, 8, 1_000)], 0, 0));
    harness.handle.update_focus(focus_now(vec![a], 0, 0));
    origin.wait_for_hits(2).await;

    origin.release_first_headers();
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        origin.hits(),
        2,
        "stale completion started a duplicate A transfer"
    );

    origin.release_body();
    wait_for_file(&harness.store.completed_path("aa11")).await;
    let stored = harness.store.read_range("aa11", 0..8).await.expect("read");
    assert_eq!(stored, Some(bytes));
    assert_eq!(origin.hits(), 2);
    std::fs::remove_dir_all(&harness.root).ok();
}

fn serial_options() -> DeliveryOptions {
    DeliveryOptions {
        params: EngineParams {
            chunk_bytes: 8,
            conservative_concurrency: 1,
            ..base_params()
        },
        level: DataUsageLevel::Conservative,
        ..DeliveryOptions::default()
    }
}
