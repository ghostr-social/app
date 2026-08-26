mod gateway_fixture;

use axum::body::to_bytes;
use axum::http::StatusCode;
use gateway_fixture::progressive::progressive_harness;
use ghostr_delivery::playback_demand::DemandState;
use ghostr_engine::ByteRange;
use tower::ServiceExt as _;

#[tokio::test]
async fn missing_bytes_emit_demand_and_stream_once_they_arrive() {
    let mut harness = progressive_harness("ghostr-progressive-demand");
    harness.posts.insert("clip");
    harness
        .bind_video("clip", "https://cdn.example/clip.mp4", Some(10))
        .await;
    harness
        .store
        .set_total_len("clip", 10)
        .await
        .expect("total length");
    harness
        .store
        .write_range("clip", 0, b"01234")
        .await
        .expect("head bytes");

    let request = harness.video_request("clip", Some("bytes=0-9")).await;
    let response = harness.router.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);

    let DemandState::Blocked(signal) = harness.demand.recv().await.expect("demand signal") else {
        panic!("first demand state must block");
    };
    assert_eq!(signal.post().as_str(), "clip");
    assert_eq!(signal.range(), ByteRange::new(5, 10));

    harness
        .store
        .write_range("clip", 5, b"56789")
        .await
        .expect("tail bytes");
    let body = to_bytes(response.into_body(), 64).await.expect("body");
    assert_eq!(&body[..], b"0123456789");
    std::fs::remove_dir_all(harness.root).expect("remove store");
}
