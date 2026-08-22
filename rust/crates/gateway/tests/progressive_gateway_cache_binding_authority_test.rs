mod gateway_fixture;

use axum::http::StatusCode;
use gateway_fixture::cache_video;
use gateway_fixture::progressive::progressive_harness;
use gateway_fixture::progressive_request::capability_request;
use ghostr_engine::{DeliveryKind, VideoMeta};
use tower::ServiceExt;

#[tokio::test]
async fn cache_replacement_retires_the_old_stored_asset() {
    let harness = progressive_harness("ghostr-progressive-cache-binding");
    let old_meta = meta("https://cdn.example/a.mp4");
    harness.bind_video("clip", &old_meta.urls[0], Some(4)).await;
    harness.store.set_total_len("clip", 4).await.unwrap();
    harness.store.write_range("clip", 0, b"aaaa").await.unwrap();
    harness.posts.replace([cache_video("clip", old_meta)]);
    let capability = harness.issue_video_asset("clip").await;

    harness
        .posts
        .replace([cache_video("clip", meta("https://cdn.example/b.mp4"))]);
    let request = capability_request("clip", capability.as_str(), None);
    let response = harness.router.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    std::fs::remove_dir_all(harness.root).expect("remove store");
}

fn meta(source: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![source.to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(4),
        duration_ms: Some(1_000),
    }
}
