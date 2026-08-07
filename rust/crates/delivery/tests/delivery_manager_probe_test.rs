//! Unknown-size posts in the window are HEAD-probed; the learned
//! length is declared to the store so the gateway can serve.

mod delivery_fixture;

use delivery_fixture::items::{focus_now, unsized_item};
use delivery_fixture::media::{hit_log, hits, media_body, serve_recording};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use delivery_fixture::wait::{wait_for_ranges, wait_total_len};

#[tokio::test]
async fn delivery_manager_probes_unknown_size_posts() {
    let log = hit_log();
    let origin = serve_recording("origin", media_body(), log.clone()).await;
    let harness = start_harness("ghostr-delivery-probe", DeliveryOptions::default());

    harness
        .handle
        .update_focus(focus_now(vec![unsized_item("aa11", &origin)], 0, 0));

    wait_total_len(&harness.store, "aa11", 16).await;
    wait_for_ranges(&harness.store, "aa11", &[(0, 16)]).await;
    assert!(
        hits(&log).iter().any(|hit| hit.starts_with("origin:HEAD:")),
        "a HEAD probe must run: {:?}",
        hits(&log)
    );
    std::fs::remove_dir_all(&harness.root).ok();
}
