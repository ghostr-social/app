mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

const URL: &str = "https://media.example/video.mp4";

#[tokio::test]
async fn newer_http_generation_revokes_a_readable_live_prefix() {
    let root = store_fixture::temp_root("live-http-generation");
    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), metadata());
    let identity = binding.transfer(URL).unwrap();
    store.bind_representation(binding).await.unwrap();
    store.select_transfer(identity.clone()).await.unwrap();
    let v1 = store_fixture::http_generation(URL, "v1", 1);
    let v2 = store_fixture::http_generation(URL, "v2", 2);
    store.apply_http_generation(&identity, v1).await.unwrap();
    let stale = store.reserve_action(&identity, 1, 8).await.unwrap();
    assert!(store
        .begin_single_response_for_action(&identity, &stale, store_fixture::exact_response(8),)
        .await
        .unwrap());
    assert!(store
        .write_single_response_for_action(&identity, &stale, 0, b"old!")
        .await
        .unwrap());
    assert_eq!(
        store.read_range("post", 0..4).await.unwrap(),
        Some(b"old!".to_vec())
    );

    store.apply_http_generation(&identity, v2).await.unwrap();

    assert!(!stale.is_active());
    assert_eq!(store.read_range("post", 0..4).await.unwrap(), None);
    assert!(!store
        .write_single_response_for_action(&identity, &stale, 4, b"old!")
        .await
        .unwrap());
    assert!(!store
        .finish_single_response_for_action(&identity, &stale, Some(8), true)
        .await
        .unwrap());
    store_fixture::discard(&root);
}

fn metadata() -> VideoMeta {
    VideoMeta {
        urls: vec![URL.into()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
