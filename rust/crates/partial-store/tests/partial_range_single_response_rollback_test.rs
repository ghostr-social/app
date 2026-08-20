mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn cancelled_replacement_keeps_the_previous_generation_readable() {
    let root = store_fixture::temp_root("single-response-rollback");
    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let transfer = binding.transfer("https://cdn.example/video").unwrap();
    let generation = SourceGeneration::try_new("https://cdn.example/video", "\"old\"", 8).unwrap();
    store.bind_representation(binding).await.unwrap();
    store.select_transfer(transfer.clone()).await.unwrap();
    store
        .accept_generation(&transfer, generation.clone())
        .await
        .unwrap();
    store
        .write_range_for_generation_if_current(&transfer, &generation, 0, b"old!")
        .await
        .unwrap();

    store
        .begin_single_response(&transfer, 3, store_fixture::exact_response(8))
        .await
        .unwrap();
    store
        .write_single_response_if_current(&transfer, 3, 0, b"new!")
        .await
        .unwrap();
    assert_eq!(
        store.read_range("post", 0..4).await.unwrap(),
        Some(b"old!".to_vec())
    );
    assert!(!store
        .finish_single_response(&transfer, 3, Some(8), false)
        .await
        .unwrap());
    assert!(store
        .write_range_for_generation_if_current(&transfer, &generation, 4, b"keep")
        .await
        .unwrap());
    assert_eq!(
        store.read_range("post", 0..8).await.unwrap(),
        Some(b"old!keep".to_vec())
    );
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
