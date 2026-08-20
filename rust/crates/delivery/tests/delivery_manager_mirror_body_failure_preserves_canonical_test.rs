mod delivery_fixture;
mod raw_http;

use delivery_fixture::gated_failure;
use delivery_fixture::items::{focus_now, seed_range};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::delivery_events::FocusItem;
use ghostr_engine::adaptive::RetrievalRequest;
use ghostr_engine::{DeliveryKind, EngineParams, PostId, VideoMeta};
use std::time::Duration;

const SHORT_206: &[u8] = b"HTTP/1.1 206 Partial Content\r\n\
Content-Type: video/mp4\r\n\
Content-Length: 4\r\n\
Content-Range: bytes 0-3/16\r\n\
ETag: \"mirror\"\r\n\
Connection: close\r\n\r\nxy";

#[tokio::test]
async fn mirror_body_failure_preserves_the_canonical_prefix() {
    let mut primary = gated_failure::serve().await;
    let (mirror, request) = raw_http::spawn_raw_server(SHORT_206).await;
    let item = mirrored_item(primary.url(), &mirror);
    let mut options = DeliveryOptions::default();
    options.params = EngineParams {
        chunk_bytes: 4,
        ..options.params
    };
    options.tuning.retry.transient_attempts = 1;
    let harness = start_harness("mirror-body-preserves", options);
    seed_range(&harness.store, &item, 0, b"0123").await;

    harness.handle.update_focus(focus_now(vec![item], 0, 5_000));
    primary.wait_started().await;
    primary.release();
    tokio::time::timeout(Duration::from_secs(2), request)
        .await
        .expect("mirror request timeout")
        .expect("mirror request task");
    tokio::task::yield_now().await;
    wait_for_independent_replan(&harness.handle, &mirror).await;

    assert_eq!(
        harness.store.read_range("post", 0..4).await.unwrap(),
        Some(b"0123".to_vec())
    );
    harness.handle.clear().await.unwrap();
    std::fs::remove_dir_all(harness.root).ok();
}

async fn wait_for_independent_replan(
    handle: &ghostr_delivery::delivery_events::DeliveryHandle,
    mirror: &str,
) {
    tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            let whole = handle.plan_history().iter().any(|evidence| {
                evidence.plan.allocations.iter().any(|allocation| {
                    allocation.source == mirror
                        && matches!(allocation.request, RetrievalRequest::FetchWhole { .. })
                })
            });
            if whole {
                return;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("incompatible mirror response replans as an independent object");
}

fn mirrored_item(primary: &str, mirror: &str) -> FocusItem {
    FocusItem {
        post: PostId::new("post"),
        meta: VideoMeta {
            urls: vec![primary.to_owned(), mirror.to_owned()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(16),
            duration_ms: Some(1_000),
        },
    }
}
