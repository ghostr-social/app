mod gateway_fixture;

use axum::body::to_bytes;
use gateway_fixture::free_space::{discard, limits, spaced_store};
use gateway_fixture::progressive::progressive_harness_with_store;
use ghostr_gateway::progressive::route::ProgressiveTiming;
use std::sync::Arc;
use tower::ServiceExt;

const VIDEO_BYTES: usize = 1_500_000;

#[tokio::test]
async fn active_gateway_stream_keeps_its_cached_file_alive() {
    let fixture = spaced_store("ghostr-gateway-lease", limits(2_000_000, 1_000), 3_000_000);
    let root = fixture.root.clone();
    let space = fixture.space.clone();
    let store = Arc::new(fixture.store);
    let harness =
        progressive_harness_with_store(root.clone(), store.clone(), ProgressiveTiming::default());
    harness.posts.insert("clip");
    store
        .set_total_len("clip", VIDEO_BYTES as u64)
        .await
        .expect("total length");
    store
        .write_range("clip", 0, &vec![7; VIDEO_BYTES])
        .await
        .expect("video bytes");
    store.finalize("clip", None).await.expect("finalize");

    let request = harness.video_request("clip", None).await;
    let response = harness.router.oneshot(request).await.expect("response");
    space.set(0);
    assert_eq!(
        store.enforce_capacity().await,
        0,
        "an active response must lease its cached file"
    );

    let body = to_bytes(response.into_body(), VIDEO_BYTES + 1)
        .await
        .expect("complete response body");
    assert_eq!(body.len(), VIDEO_BYTES);
    assert_eq!(store.enforce_capacity().await, VIDEO_BYTES as u64);
    discard(&root);
}
