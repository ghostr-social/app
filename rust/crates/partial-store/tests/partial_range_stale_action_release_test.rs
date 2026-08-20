mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::ResponseOpenResult;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn stale_same_id_action_cannot_abort_the_new_response_authority() {
    let root = store_fixture::temp_root("stale-action-release");
    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let identity = binding.transfer("https://cdn.example/video").unwrap();
    store.bind_representation(binding).await.unwrap();
    store.select_transfer(identity.clone()).await.unwrap();
    store.write_range("post", 0, b"old!").await.unwrap();
    let stale = store.reserve_action(&identity, 1, 8).await.unwrap();
    assert_eq!(
        store
            .open_single_response_for_action(&identity, &stale, store_fixture::exact_response(8),)
            .await
            .unwrap(),
        ResponseOpenResult::Opened
    );
    store.release_action(&stale).await;
    let current = store.reserve_action(&identity, 1, 8).await.unwrap();
    assert_eq!(
        store
            .open_single_response_for_action(&identity, &current, store_fixture::exact_response(8),)
            .await
            .unwrap(),
        ResponseOpenResult::Opened
    );

    store.release_action(&stale).await;

    assert!(store
        .write_single_response_for_action(&identity, &current, 0, b"new!")
        .await
        .unwrap());
    store.release_action(&current).await;
    store_fixture::discard(&root);
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
