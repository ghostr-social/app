mod delivery_fixture;
mod hls_terminal_wait;
mod raw_http;

use core::time::Duration;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::segmented::SegmentedPhase;
use ghostr_engine::DeliveryKind;
use hls_terminal_wait::wait_terminal;

const MANIFEST: &[u8] = b"HTTP/1.1 200 OK\r\n\
Content-Type: application/vnd.apple.mpegurl\r\n\
Content-Length: 70\r\nConnection: close\r\n\r\n\
#EXTM3U\n#EXT-X-TARGETDURATION:4\n#EXTINF:4,\nsegment.m4s\n#EXT-X-ENDLIST\n";
const PARTIAL_SEGMENT: &[u8] = b"HTTP/1.1 206 Partial Content\r\n\
Content-Type: video/iso.segment\r\n\
Content-Length: 4\r\nContent-Range: bytes 0-3/7\r\n\
Connection: close\r\n\r\nsegm";

#[tokio::test]
async fn partial_hls_object_cannot_be_cached_as_complete_and_ready() {
    let (source, requests) =
        raw_http::spawn_response_sequence(vec![MANIFEST, PARTIAL_SEGMENT]).await;
    let harness = start_harness("hls-partial-object", DeliveryOptions::default());
    let mut item = sized_item("stream", &source, 70, 4_000);
    item.meta.delivery = DeliveryKind::Hls;
    harness.handle.update_focus(focus_now(vec![item], 0, 0));

    let snapshot = wait_terminal(&harness.segmented, "stream").await;
    tokio::time::timeout(Duration::from_secs(2), requests)
        .await
        .expect("manifest and segment requests")
        .expect("origin server");

    assert_eq!(snapshot.phase, SegmentedPhase::Failed);
    assert_eq!(snapshot.bytes_present, 0);
    assert!(harness.segmented.object(&source).is_none());
    std::fs::remove_dir_all(&harness.root).ok();
}
