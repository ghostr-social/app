mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn strong_generation_survives_restart_for_if_range_continuation() {
    let root = store_fixture::temp_root("partial-generation-restart");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("same"), meta());
    let transfer = binding.transfer("https://a.example/video").unwrap();
    let generation =
        SourceGeneration::try_new("https://cdn.example/video", "\"version-one\"", 8).unwrap();
    let first = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    first.bind_representation(binding.clone()).await.unwrap();
    first.select_transfer(transfer.clone()).await.unwrap();
    first
        .accept_generation(&transfer, generation.clone())
        .await
        .unwrap();
    assert!(first
        .write_range_for_generation_if_current(&transfer, &generation, 0, b"part")
        .await
        .unwrap());
    drop(first);

    let reopened = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.unwrap();
    reopened.bind_representation(binding).await.unwrap();
    let restored = reopened.select_transfer(transfer).await.unwrap();

    assert_eq!(restored, Some(generation));
    assert_eq!(
        reopened.read_range("same", 0..4).await.unwrap(),
        Some(b"part".to_vec())
    );
    store_fixture::discard(&root);
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://a.example/video".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
