mod gateway_fixture;

use axum::body::to_bytes;
use axum::http::StatusCode;
use gateway_fixture::progressive::progressive_harness;
use gateway_fixture::progressive_request::capability_request;
use ghostr_engine::adaptive::WholeBodyContract;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use tower::ServiceExt;

#[tokio::test]
async fn old_asset_cannot_serve_a_new_revision_of_the_same_representation() {
    let harness = progressive_harness("ghostr-progressive-asset-revision");
    harness.posts.insert("clip");
    let transfer = install_initial(&harness).await;
    let old = harness.issue_video_asset("clip").await;
    replace_bytes(&harness, transfer).await;
    let new = harness.issue_video_asset("clip").await;
    assert_assets(&harness, old.as_str(), new.as_str()).await;
    std::fs::remove_dir_all(harness.root).expect("remove store");
}

async fn install_initial(
    harness: &gateway_fixture::progressive::ProgressiveHarness,
) -> ghostr_engine::representation::TransferIdentity {
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta());
    let transfer = binding.transfer("https://cdn.example/clip.mp4").unwrap();
    harness.store.bind_representation(binding).await.unwrap();
    harness.store.set_total_len("clip", 4).await.unwrap();
    harness.store.write_range("clip", 0, b"aaaa").await.unwrap();
    transfer
}

async fn replace_bytes(
    harness: &gateway_fixture::progressive::ProgressiveHarness,
    transfer: ghostr_engine::representation::TransferIdentity,
) {
    harness
        .store
        .select_transfer(transfer.clone())
        .await
        .unwrap();
    let contract = WholeBodyContract::Exact { expected_bytes: 4 };
    harness
        .store
        .begin_single_response(&transfer, 7, contract)
        .await
        .unwrap();
    harness
        .store
        .write_single_response_if_current(&transfer, 7, 0, b"bbbb")
        .await
        .unwrap();
    harness
        .store
        .finish_single_response(&transfer, 7, Some(4), true)
        .await
        .unwrap();
}

async fn assert_assets(
    harness: &gateway_fixture::progressive::ProgressiveHarness,
    old: &str,
    new: &str,
) {
    let stale = capability_request("clip", old, None);
    let stale = harness.router.clone().oneshot(stale).await.unwrap();
    assert_eq!(stale.status(), StatusCode::NOT_FOUND);
    let current = capability_request("clip", new, None);
    let current = harness.router.clone().oneshot(current).await.unwrap();
    assert_eq!(to_bytes(current.into_body(), 4).await.unwrap(), b"bbbb"[..]);
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://cdn.example/clip.mp4".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(4),
        duration_ms: Some(1_000),
    }
}
