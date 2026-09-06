//! A stale advertised size must not repeat the same header-only whole probe.

mod delivery_fixture;

use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, hits};
use delivery_fixture::options::production_geometry_parallel_options;
use delivery_fixture::probe_origins::serve_header_bound_then_complete;
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_engine::adaptive::{BOOTSTRAP_DIRECT_FETCH_BYTES, REQUEST_SLICE_BYTES};

#[tokio::test]
async fn conflicting_advertised_size_does_not_repeat_an_exhausted_header_probe() {
    let total = BOOTSTRAP_DIRECT_FETCH_BYTES + REQUEST_SLICE_BYTES;
    let body = vec![b'b'; total as usize];
    let log = hit_log();
    let origin = serve_header_bound_then_complete(std::sync::Arc::clone(&log), body.clone()).await;
    let harness = start_harness(
        "ghostr-conflicting-header-bound",
        production_geometry_parallel_options(),
    );
    let item = sized_item("aa11", &origin, BOOTSTRAP_DIRECT_FETCH_BYTES, 10_000);
    harness.handle.update_focus(focus_now(vec![item], 0, 0));

    let ready = tokio::time::timeout(
        core::time::Duration::from_secs(9),
        wait_for_ranges(&harness.store, "aa11", &[(0, total)]),
    )
    .await;
    let requests = hits(&log);
    assert!(
        ready.is_ok(),
        "header recovery did not complete: {requests:?}"
    );
    assert_eq!(
        harness
            .store
            .read_range("aa11", 0..total)
            .await
            .expect("valid stored response"),
        Some(body)
    );
    assert_eq!(
        requests
            .iter()
            .filter(|request| *request == "GET:full")
            .count(),
        2,
        "header discovery must replan once: {requests:?}"
    );
    std::fs::remove_dir_all(&harness.root).ok();
}
