mod delivery_fixture;
mod raw_http;

use core::time::Duration;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::segmented::SegmentedPhase;
use ghostr_engine::DeliveryKind;

#[tokio::test]
async fn invalid_hls_manifest_surfaces_a_failed_readiness_state() {
    let response = b"HTTP/1.1 200 OK\r\nContent-Type: application/vnd.apple.mpegurl\r\nContent-Length: 8\r\nConnection: close\r\n\r\n#EXTM3U\n";
    let (source, request) = raw_http::spawn_raw_server(response).await;
    let source = format!("{source}?token=do-not-project-this-secret");
    let harness = start_harness("hls-invalid-manifest", DeliveryOptions::default());
    let mut item = sized_item("stream", &source, 32, 4_000);
    item.meta.delivery = DeliveryKind::Hls;
    harness.handle.update_focus(focus_now(vec![item], 0, 0));
    request.await.expect("valid test fixture");

    let changed = harness.segmented.notifier();
    tokio::time::timeout(Duration::from_secs(2), async {
        while harness.segmented.snapshot("stream").phase != SegmentedPhase::Failed {
            changed.notified().await;
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
