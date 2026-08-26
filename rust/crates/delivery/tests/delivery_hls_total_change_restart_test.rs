mod delivery_fixture;
mod hls_terminal_wait;

use delivery_fixture::bounded_hls_generation::{
    serve_total_change, CHANGED_TOTAL_BYTES, INIT_BYTES,
};
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::segmented::SegmentedPhase;
use ghostr_engine::DeliveryKind;

#[tokio::test]
async fn changed_content_range_total_restarts_before_appending_bytes() {
    let (source, requests) = serve_total_change().await;
    let mut options = DeliveryOptions::default();
    options.params.chunk_bytes = 256 * 1024;
    let harness = start_harness("hls-total-change", options);
    let mut item = sized_item("stream", &source, INIT_BYTES as u64, 4_000);
    item.meta.delivery = DeliveryKind::Hls;
    harness.handle.update_focus(focus_now(vec![item], 0, 0));

    let terminal = hls_terminal_wait::wait_terminal(&harness.segmented, "stream").await;

    assert_eq!(terminal.phase, SegmentedPhase::Ready);
    assert_eq!(requests.lock().expect("valid test fixture").len(), 4);
    let init = harness
        .segmented
        .object(&source.replace("index.m3u8", "init.mp4"))
        .expect("valid test fixture");
    assert_eq!(init.body.as_ref(), vec![8; CHANGED_TOTAL_BYTES]);
    std::fs::remove_dir_all(&harness.root).ok();
}
