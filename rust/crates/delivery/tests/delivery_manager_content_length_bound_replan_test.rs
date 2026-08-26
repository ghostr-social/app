//! A header-discovered bound replans the same representation as an exact whole fetch.

mod delivery_fixture;

use delivery_fixture::evidence::DeliveryEvidence as _;
use delivery_fixture::items::{focus_now, unsized_item};
use delivery_fixture::media::{hit_log, hits};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::probe_origins::serve_header_bound_then_complete;
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_engine::adaptive::REQUEST_SLICE_BYTES;

#[tokio::test]
async fn oversized_header_replans_once_and_then_completes_exactly() {
    let body = vec![b'b'; REQUEST_SLICE_BYTES as usize + 1];
    let log = hit_log();
    let origin = serve_header_bound_then_complete(std::sync::Arc::clone(&log), body.clone()).await;
    let harness = start_harness("ghostr-header-bound-replan", DeliveryOptions::default());

    harness
        .handle
        .update_focus(focus_now(vec![unsized_item("aa11", &origin)], 0, 0));

    wait_for_ranges(&harness.store, "aa11", &[(0, body.len() as u64)]).await;
    assert_eq!(
        harness
            .store
            .read_range("aa11", 0..body.len() as u64)
            .await
            .expect("valid test fixture"),
        Some(body)
    );
    let requests = hits(&log);
    assert_eq!(
        requests
            .iter()
            .filter(|request| *request == "GET:full")
            .count(),
        2,
        "header discovery retried or failed to replan: {requests:?}"
    );
    assert_eq!(
        harness
            .handle
            .evaluation_snapshot()
            .efficiency
            .aborted_bytes,
        0
    );
    std::fs::remove_dir_all(&harness.root).ok();
}
