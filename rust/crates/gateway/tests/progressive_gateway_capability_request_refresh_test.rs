mod gateway_fixture;

use axum::http::StatusCode;
use gateway_fixture::progressive::progressive_harness;
use std::time::Duration;
use tower::ServiceExt;

#[tokio::test(start_paused = true)]
async fn valid_request_refreshes_capability_before_waiting_for_length() {
    let harness = progressive_harness("ghostr-capability-request-refresh");
    harness.posts.insert("clip");
    harness
        .bind_video("clip", "https://cdn.example/clip.mp4", None)
        .await;
    let request = harness.video_request("clip", None).await;
    tokio::time::advance(Duration::from_secs(30 * 60 - 1)).await;

    let store = harness.store.clone();
    let learned = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(1_500)).await;
        store.set_total_len("clip", 4).await.unwrap();
    });
    let response = harness.router.clone().oneshot(request).await.unwrap();
    learned.await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    std::fs::remove_dir_all(harness.root).expect("remove store");
}
