mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn clear_revokes_actions_and_releases_their_capacity() {
    let fixture = store_fixture::spaced_store("clear-actions", store_fixture::limits(8, 0), 8);
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let identity = binding.transfer("https://cdn.example/video").unwrap();
    fixture
        .store
        .bind_representation(binding.clone())
        .await
        .unwrap();
    let stale = fixture.store.reserve_action(&identity, 1, 8).await.unwrap();
    fixture
        .store
        .open_single_response_for_action(&identity, &stale, store_fixture::exact_response(8))
        .await
        .unwrap();
    fixture
        .store
        .write_single_response_for_action(&identity, &stale, 0, b"half")
        .await
        .unwrap();

    fixture.store.clear().await.unwrap();

    assert!(!stale.is_active());
    fixture.store.bind_representation(binding).await.unwrap();
    let current = fixture.store.reserve_action(&identity, 2, 8).await.unwrap();
    assert!(!fixture
        .store
        .write_single_response_for_action(&identity, &stale, 4, b"late")
        .await
        .unwrap());
    fixture.store.release_action(&current).await;
    store_fixture::discard(&fixture.root);
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://cdn.example/video".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
