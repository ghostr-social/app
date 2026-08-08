//! A stale A1 completion cannot evict the newer A2 transfer grant.

mod delivery_fixture;
mod range_fixture;

use delivery_fixture::aba_origin::serve;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::{base_params, DeliveryOptions};
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_file;
use ghostr_engine::{DataUsageLevel, EngineParams};
use range_fixture::stall::serve_stalling;
use std::time::Duration;

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
    wait_for_file(&harness.root.join("aa11.video")).await;
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
