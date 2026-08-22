mod gateway_fixture;

use axum::body::to_bytes;
use axum::http::StatusCode;
use gateway_fixture::progressive::progressive_harness;
use gateway_fixture::progressive_request::capability_request;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use tower::ServiceExt;

#[tokio::test]
async fn old_progressive_asset_cannot_serve_a_replacement_representation() {
    let harness = progressive_harness("ghostr-progressive-asset-authority");
    harness.posts.insert("clip");
    bind_bytes(&harness, "https://cdn.example/a.mp4", b"aaaa").await;
    let old = harness.issue_video_asset("clip").await;

    bind_bytes(&harness, "https://cdn.example/b.mp4", b"bbbb").await;
    let new = harness.issue_video_asset("clip").await;

    let stale = capability_request("clip", old.as_str(), None);
    let stale = harness.router.clone().oneshot(stale).await.unwrap();
    assert_eq!(stale.status(), StatusCode::NOT_FOUND);
    assert_ne!(old, new);

    let current = capability_request("clip", new.as_str(), None);
    let current = harness.router.clone().oneshot(current).await.unwrap();
    assert_eq!(current.status(), StatusCode::OK);
    assert_eq!(to_bytes(current.into_body(), 4).await.unwrap(), b"bbbb"[..]);
    std::fs::remove_dir_all(harness.root).expect("remove store");
}

async fn bind_bytes(
    harness: &gateway_fixture::progressive::ProgressiveHarness,
    source: &str,
    bytes: &[u8],
) {
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta(source, bytes.len() as u64));
    harness.store.bind_representation(binding).await.unwrap();
    harness
        .store
        .set_total_len("clip", bytes.len() as u64)
        .await
        .unwrap();
    harness.store.write_range("clip", 0, bytes).await.unwrap();
}

fn meta(source: &str, size: u64) -> VideoMeta {
    VideoMeta {
        urls: vec![source.to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(size),
        duration_ms: Some(1_000),
    }
}
