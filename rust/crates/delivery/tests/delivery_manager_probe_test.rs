//! Unknown-size posts in the window are HEAD-probed; the learned
//! length is declared to the store so the gateway can serve.

mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::items::{focus_now, unsized_item};
use delivery_fixture::media::{hit_log, hits, media_body, serve_recording, HitLog};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use delivery_fixture::wait::{wait_for_ranges, wait_total_len};
use ghostr_engine::EngineParams;

#[tokio::test]
async fn delivery_manager_probes_unknown_size_posts() {
    let log = hit_log();
    let origin = serve_recording("origin", media_body(), std::sync::Arc::clone(&log)).await;
    let mut options = DeliveryOptions::default();
    options.params = EngineParams {
        chunk_bytes: 4,
        ..options.params
    };
    let harness = start_harness("ghostr-delivery-probe", options);

    harness
        .handle
        .update_focus(focus_now(vec![unsized_item("aa11", &origin)], 0, 0));

    let requests = wait_for_head_and_body(&log).await;
    assert_eq!(requests[0], "origin:HEAD:full");
    assert_eq!(requests[1], "origin:GET:0-3");
    wait_total_len(&harness.store, "aa11", 16).await;
    wait_for_ranges(&harness.store, "aa11", &[(0, 16)]).await;
    assert_eq!(
        hits(&log)
            .iter()
            .filter(|hit| hit.contains(":HEAD:"))
            .count(),
        1
    );
    std::fs::remove_dir_all(&harness.root).ok();
}

async fn wait_for_head_and_body(log: &HitLog) -> Vec<String> {
    let observed = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let requests = hits(log);
            if requests.len() >= 2 {
                return requests;
            }
            tokio::task::yield_now().await;
        }
    })
    .await;
    assert!(
        observed.is_ok(),
        "HEAD did not converge to body: {:?}",
        hits(log)
    );
    observed.expect("valid test fixture")
}
