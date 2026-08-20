#![cfg(unix)]

mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::ResponseOpenResult;

#[tokio::test]
async fn failed_stage_cleanup_releases_future_capacity_and_retries_owned_bytes() {
    let fixture =
        store_fixture::spaced_store("action-cleanup-failure", store_fixture::limits(12, 0), 12);
    let mut catalog = Catalog::new();
    let first_binding = catalog.upsert(PostId::new("post"), meta("post"));
    let first = first_binding.transfer("https://cdn.example/post").unwrap();
    fixture
        .store
        .bind_representation(first_binding)
        .await
        .unwrap();
    fixture.store.select_transfer(first.clone()).await.unwrap();
    fixture.store.write_range("post", 0, b"old!").await.unwrap();
    let action = fixture.store.reserve_action(&first, 1, 8).await.unwrap();
    assert_eq!(
        fixture
            .store
            .open_single_response_for_action(&first, &action, store_fixture::exact_response(8),)
            .await
            .unwrap(),
        ResponseOpenResult::Opened
    );
    fixture
        .store
        .write_single_response_for_action(&first, &action, 0, b"new!")
        .await
        .unwrap();
    std::fs::create_dir(fixture.root.join("post.response.commit")).unwrap();
    fixture.store.release_action(&action).await;

    let second_binding = catalog.upsert(PostId::new("other"), meta("other"));
    let second = second_binding
        .transfer("https://cdn.example/other")
        .unwrap();
    fixture
        .store
        .bind_representation(second_binding)
        .await
        .unwrap();
    let other = fixture.store.reserve_action(&second, 2, 4).await.unwrap();
    fixture.store.release_action(&other).await;
    std::fs::remove_dir(fixture.root.join("post.response.commit")).unwrap();

    let retry = fixture.store.reserve_action(&first, 3, 8).await.unwrap();
    assert_eq!(*fixture.used_bytes.lock().await, 4);
    fixture.store.release_action(&retry).await;
    store_fixture::discard(&fixture.root);
}

fn meta(name: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![format!("https://cdn.example/{name}")],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
