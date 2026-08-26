//! A successful response without a usable length does not end probing.

mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::items::{focus_now, unsized_mirrored_item};
use delivery_fixture::media::{hit_log, hits, media_body, serve_recording};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::probe_origins::serve_lengthless;
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_total_len;

#[tokio::test]
async fn delivery_manager_falls_back_after_a_lengthless_probe() {
    let lengthless = serve_lengthless().await;
    let log = hit_log();
    let mirror = serve_recording("mirror", media_body(), std::sync::Arc::clone(&log)).await;
    let mut options = DeliveryOptions::default();
    options.tuning.retry.transient_attempts = 1;
    options.params.conservative_concurrency = 0;
    options.params.balanced_concurrency = 0;
    options.params.aggressive_concurrency = 0;
    let harness = start_harness("ghostr-lengthless-probe", options);
    let item = unsized_mirrored_item("aa11", &lengthless, &mirror);

    harness.handle.update_focus(focus_now(vec![item], 0, 0));

    let recovered = tokio::time::timeout(
        Duration::from_secs(2),
        wait_total_len(&harness.store, "aa11", 16),
    )
    .await;
    assert!(
        recovered.is_ok(),
        "probe never reached mirror: {:?}",
        hits(&log)
    );
    std::fs::remove_dir_all(&harness.root).ok();
}
