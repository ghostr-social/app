mod delivery_fixture;
mod raw_http;

use delivery_fixture::hls::{serve, HlsGate};
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::segmented::SegmentedPhase;
use ghostr_engine::DeliveryKind;
use std::time::Duration;

#[tokio::test]
async fn invalid_first_hls_source_falls_back_to_the_next_mirror() {
    let response = b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 3\r\nConnection: close\r\n\r\nbad";
    let (invalid, request) = raw_http::spawn_raw_server(response).await;
    let gate = HlsGate::new();
    let valid = serve(gate.clone()).await;
    let harness = start_harness("hls-mirror-fallback", DeliveryOptions::default());
    let mut item = sized_item("stream", &invalid, 32, 4_000);
    item.meta.delivery = DeliveryKind::Hls;
    item.meta.urls.push(valid);
    harness.handle.update_focus(focus_now(vec![item], 0, 0));

    request.await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), gate.started.acquire())
        .await
        .unwrap()
        .unwrap()
        .forget();
    gate.release.add_permits(1);
    wait_ready(&harness).await;

    std::fs::remove_dir_all(&harness.root).ok();
}

async fn wait_ready(harness: &delivery_fixture::DeliveryHarness) {
    let changed = harness.segmented.notifier();
    tokio::time::timeout(Duration::from_secs(2), async {
        while harness.segmented.snapshot("stream").phase != SegmentedPhase::Ready {
            changed.notified().await;
        }
    })
    .await
    .unwrap();
}
