mod support;

use std::time::Duration;
use support::delivery::start_harness;
use support::delivery_items::candidate;
use support::delivery_options::DeliveryOptions;
use support::delivery_probe_origins::serve_lengthless;

#[tokio::test]
async fn a_candidate_stays_registered_while_metadata_is_pending() {
    let origin = serve_lengthless().await;
    let harness = start_harness("ghostr-candidate-cache", DeliveryOptions::default());

    harness
        .handle
        .admit_candidate(candidate("aa11", &origin, None, 42));

    tokio::time::timeout(Duration::from_secs(1), async {
        while !harness.cache.contains("aa11") {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("pending candidate should remain gateway-visible");
    std::fs::remove_dir_all(&harness.root).ok();
}
