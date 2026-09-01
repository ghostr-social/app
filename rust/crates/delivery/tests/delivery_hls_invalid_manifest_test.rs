mod delivery_fixture;
mod raw_http;

use core::time::Duration;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::segmented::SegmentedPhase;
use ghostr_engine::DeliveryKind;

const WAIT_LIMIT: Duration = Duration::from_secs(10);

#[tokio::test]
async fn invalid_hls_manifest_retries_then_surfaces_a_failed_readiness_state() {
    let response = b"HTTP/1.1 200 OK\r\nContent-Type: application/vnd.apple.mpegurl\r\nContent-Length: 8\r\nConnection: close\r\n\r\n#EXTM3U\n";
    let (source, requests) = raw_http::spawn_response_sequence(vec![response, response]).await;
    let source = format!("{source}?token=do-not-project-this-secret");
    let mut options = DeliveryOptions::default();
    options.tuning.retry.permanent_attempts = 2;
    let harness = start_harness("hls-invalid-manifest", options);
    let mut item = sized_item("stream", &source, 32, 4_000);
    item.meta.delivery = DeliveryKind::Hls;
    harness.handle.update_focus(focus_now(vec![item], 0, 0));
    tokio::time::timeout(WAIT_LIMIT, requests)
        .await
        .expect("invalid manifest retry")
        .expect("valid test fixture");

    let changed = harness.segmented.notifier();
    tokio::time::timeout(WAIT_LIMIT, async {
        loop {
            let changed = changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if harness.segmented.snapshot("stream").phase == SegmentedPhase::Failed {
                return;
            }
            changed.await;
        }
    })
    .await
    .expect("valid test fixture");

    let detail = harness
        .segmented
        .snapshot("stream")
        .detail
        .expect("safe HLS failure detail");
    assert_eq!(detail, "HLS bootstrap received an invalid response");
    assert!(!detail.contains("do-not-project-this-secret"));
    std::fs::remove_dir_all(&harness.root).ok();
}
