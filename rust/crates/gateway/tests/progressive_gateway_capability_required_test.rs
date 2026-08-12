mod gateway_fixture;

use axum::http::StatusCode;
use gateway_fixture::progressive::progressive_harness;
use gateway_fixture::progressive_request::video_request;
use tower::ServiceExt;

#[tokio::test]
async fn post_id_alone_does_not_authorize_playback() {
    let harness = progressive_harness("ghostr-progressive-capability-required");
    harness.posts.insert("clip");
    harness
        .store
        .set_total_len("clip", 1)
        .await
        .expect("total length");
    harness
        .store
        .write_range("clip", 0, b"x")
        .await
        .expect("bytes");

    let response = harness
        .router
        .oneshot(video_request("clip", None))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    std::fs::remove_dir_all(harness.root).expect("remove store");
}
