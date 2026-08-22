mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn short_live_response_is_discarded_before_a_retry() {
    let root = store_fixture::temp_root("single-response-short");
    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let transfer = binding.transfer("https://cdn.example/video").unwrap();
    store.bind_representation(binding).await.unwrap();
    store.select_transfer(transfer.clone()).await.unwrap();
    assert!(store
        .begin_single_response(&transfer, 1, store_fixture::exact_response(8))
        .await
        .unwrap());
    store
        .write_single_response_if_current(&transfer, 1, 0, b"short")
        .await
        .unwrap();

    store
        .finish_single_response(&transfer, 1, Some(8), true)
        .await
        .expect_err("framed response ended before the declared length");

    assert_eq!(store.read_range("post", 0..5).await.unwrap(), None);
    assert!(!store
        .begin_single_response(&transfer, 2, store_fixture::exact_response(8))
        .await
        .unwrap());
    store.select_transfer(transfer.clone()).await.unwrap();
    assert!(store
        .begin_single_response(&transfer, 2, store_fixture::exact_response(8))
        .await
        .unwrap());
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
