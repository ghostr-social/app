mod delivery_fixture;
mod raw_http;

use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::delivery_events::DeliveryHandle;
use ghostr_engine::adaptive::RetrievalRequest;
use ghostr_engine::EngineParams;
use std::time::Duration;

#[tokio::test]
async fn coherent_validatorless_206_replans_as_one_independent_object() {
    let head = b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: 16\r\nAccept-Ranges: bytes\r\nConnection: close\r\n\r\n";
    let body = b"HTTP/1.1 206 Partial Content\r\nContent-Type: video/mp4\r\nContent-Length: 4\r\nContent-Range: bytes 0-3/16\r\n\r\n01";
    let origin = raw_http::spawn_split_response(head, body, b"23").await;
    let item = sized_item("post", &origin.url, 16, 1_000);
    let mut options = DeliveryOptions::default();
    options.params = EngineParams {
        chunk_bytes: 4,
        ..options.params
    };
    let harness = start_harness("validatorless-range", options);

    harness.handle.update_focus(focus_now(vec![item], 0, 5_000));
    origin.prefix_sent.await.expect("206 response opened");
    wait_for_whole_plan(&harness.handle, &origin.url).await;

    origin.release.notify_one();
    origin.requests.await.unwrap();
    harness.handle.clear().await.unwrap();
    std::fs::remove_dir_all(harness.root).ok();
}

async fn wait_for_whole_plan(handle: &DeliveryHandle, source: &str) {
    let result = tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            let planned =
                handle.plan_history().iter().any(|evidence| {
                    evidence.plan.allocations.iter().any(|allocation| {
                        whole(allocation.source.as_str(), allocation.request, source)
                    }) || evidence.plan.retained.iter().any(|allocation| {
                        whole(allocation.source.as_str(), allocation.request, source)
                    })
                });
            if planned {
                return;
            }
            tokio::task::yield_now().await;
        }
    })
    .await;
    assert!(
        result.is_ok(),
        "validatorless sparse response did not trigger an independent whole plan: {:?}",
        handle.plan_history()
    );
}

fn whole(observed: &str, request: RetrievalRequest, expected: &str) -> bool {
    observed == expected && matches!(request, RetrievalRequest::FetchWhole { .. })
}
