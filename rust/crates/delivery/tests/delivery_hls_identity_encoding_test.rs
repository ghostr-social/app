mod delivery_fixture;
mod raw_http;

use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::{start_harness, DeliveryHarness};
use ghostr_delivery::segmented::{SegmentedPhase, SegmentedSnapshot};
use ghostr_engine::DeliveryKind;
use std::time::Duration;

const MANIFEST: &[u8] = b"HTTP/1.1 200 OK\r\n\
Content-Type: application/vnd.apple.mpegurl\r\n\
Content-Length: 70\r\n\
Connection: close\r\n\r\n\
#EXTM3U\n#EXT-X-TARGETDURATION:4\n#EXTINF:4,\nsegment.m4s\n#EXT-X-ENDLIST\n";
const CODED_SEGMENT: &[u8] = b"HTTP/1.1 200 OK\r\n\
Content-Type: video/iso.segment\r\n\
Content-Length: 7\r\n\
Content-Encoding: identity\r\n\
Content-Encoding: gzip\r\n\
Connection: close\r\n\r\nsegment";

#[tokio::test]
async fn coded_hls_segment_cannot_become_ready_or_cacheable() {
    let responses = vec![MANIFEST, CODED_SEGMENT];
    let (source, requests) = raw_http::spawn_response_sequence(responses).await;
    let harness = start_harness("hls-coded-segment", DeliveryOptions::default());
    let mut item = sized_item("stream", &source, 70, 4_000);
    item.meta.delivery = DeliveryKind::Hls;
    harness.handle.update_focus(focus_now(vec![item], 0, 0));

    let snapshot = wait_terminal(&harness).await;
    assert_eq!(snapshot.phase, SegmentedPhase::Failed);
    assert_eq!(snapshot.bytes_present, 0);
    assert!(harness.segmented.object(&source).is_none());
    requests.await.expect("origin requests");
    std::fs::remove_dir_all(&harness.root).ok();
}

async fn wait_terminal(harness: &DeliveryHarness) -> SegmentedSnapshot {
    let changed = harness.segmented.notifier();
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let notified = changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let snapshot = harness.segmented.snapshot("stream");
            if matches!(
                snapshot.phase,
                SegmentedPhase::Ready | SegmentedPhase::Failed
            ) {
                return snapshot;
            }
            notified.await;
        }
    })
    .await
    .expect("terminal readiness")
}
