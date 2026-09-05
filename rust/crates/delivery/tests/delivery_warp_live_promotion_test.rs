//! Unopened requests cannot create promotion authority or suppress whole fallback.
mod delivery_fixture;
mod raw_http;

use delivery_fixture::evidence::DeliveryEvidence as _;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_file;
use ghostr_engine::adaptive::RecordedWarpCommand;
use raw_http::spawn_gated_response;

const PROBE: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 16\r\nConnection: close\r\n\r\n";
const WHOLE: &[u8] = b"HTTP/1.1 200 OK\r\nCache-Control: public, max-age=3600\r\nETag: \"stable\"\r\nContent-Type: video/mp4\r\nContent-Length: 16\r\nConnection: close\r\n\r\n0123456789abcdef";

#[tokio::test]
async fn cold_current_cannot_promote_before_response_headers() {
    let origin = spawn_gated_response(PROBE, WHOLE).await;
    let harness = start_harness("warp-live-promotion", DeliveryOptions::default());
    harness.handle.update_focus(focus_now(
        vec![sized_item("post", &origin.url, 16, 1_000)],
        0,
        5_000,
    ));
    let request = origin.body_request.await.expect("whole request");
    assert!(request.starts_with(b"GET "), "current starts with body GET");
    let history = harness.handle.decision_history();
    assert!(
        history.records.iter().all(|record| {
            !matches!(
                record
                    .warp_decision
                    .as_ref()
                    .and_then(|warp| warp.selected.as_ref())
                    .map(|selected| &selected.command),
                Some(RecordedWarpCommand::Promote { .. })
            )
        }),
        "headers must establish any promotion opportunity"
    );
    origin.release_headers.notify_one();
    origin.requests.await.expect("one body request completes");
    wait_for_file(&harness.root.join("post.video")).await;
    assert_eq!(
        std::fs::read(harness.root.join("post.video")).expect("cached body"),
        b"0123456789abcdef"
    );
    harness.handle.clear().await.expect("clear delivery");
    std::fs::remove_dir_all(&harness.root).ok();
}
