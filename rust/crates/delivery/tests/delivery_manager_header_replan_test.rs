mod delivery_fixture;
mod raw_http;

use core::time::Duration;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::delivery_events::DeliveryHandle;
use ghostr_engine::{EngineParams, PostId};
use raw_http::spawn_split_response;

#[tokio::test]
async fn coherent_206_replans_before_the_first_body_finishes() {
    let head = b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: 16\r\nAccept-Ranges: bytes\r\nConnection: close\r\n\r\n";
    let body = b"HTTP/1.1 206 Partial Content\r\nContent-Type: video/mp4\r\nContent-Length: 4\r\nContent-Range: bytes 0-3/16\r\nETag: \"same\"\r\n\r\n01";
    let origin = spawn_split_response(head, body, b"23").await;
    let item = sized_item("post", &origin.url, 16, 1_000);
    let mut options = DeliveryOptions::default();
    options.params = EngineParams {
        chunk_bytes: 4,
        ..options.params
    };
    let harness = start_harness("header-replan", options);

    harness.handle.update_focus(focus_now(vec![item], 0, 5_000));
    origin.prefix_sent.await.expect("first response prefix");
    wait_for_followup_plan(&harness.handle).await;

    origin.release.notify_one();
    origin.requests.await.expect("first response completion");
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(harness.root).ok();
}

async fn wait_for_followup_plan(handle: &DeliveryHandle) {
    tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            let followup = handle.plan_history().iter().any(|evidence| {
                evidence.plan.allocations.iter().any(|work| {
                    work.post == PostId::new("post") && work.request.requested_bytes().start > 0
                })
            });
            if followup {
                return;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("response header triggers a follow-up plan before body EOF");
}
