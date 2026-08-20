mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn sparse_reacceptance_cannot_relabel_live_whole_bytes() {
    let root = store_fixture::temp_root("partial-live-whole-takeover");
    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let transfer = binding.transfer(&meta().urls[0]).unwrap();
    let generation = SourceGeneration::try_new(&meta().urls[0], "\"one\"", 8).unwrap();
    store.bind_representation(binding).await.unwrap();
    store.select_transfer(transfer.clone()).await.unwrap();
    store
        .accept_generation(&transfer, generation.clone())
        .await
        .unwrap();
    store
        .begin_single_response(&transfer, 1, store_fixture::exact_response(8))
        .await
        .unwrap();
    store
        .write_single_response_if_current(&transfer, 1, 0, b"whole")
        .await
        .unwrap();

    store
        .accept_generation(&transfer, generation.clone())
        .await
        .unwrap();

    assert_eq!(store.read_range("post", 0..5).await.unwrap(), None);
    assert!(store
        .write_range_for_generation_if_current(&transfer, &generation, 0, b"range")
        .await
        .unwrap());
    assert_eq!(
        store.read_range("post", 0..5).await.unwrap(),
        Some(b"range".to_vec())
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
