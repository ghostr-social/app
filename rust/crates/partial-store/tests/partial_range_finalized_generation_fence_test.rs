mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn late_generation_cannot_replace_a_finalized_video() {
    let root = store_fixture::temp_root("partial-finalized-generation");
    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let transfer = binding.transfer(&meta().urls[0]).unwrap();
    let mirror = binding.transfer(&meta().urls[1]).unwrap();
    let first = generation("\"one\"");
    store.bind_representation(binding.clone()).await.unwrap();
    store.select_transfer(transfer.clone()).await.unwrap();
    store
        .accept_generation(&transfer, first.clone())
        .await
        .unwrap();
    store
        .write_range_for_generation_if_current(&transfer, &first, 0, b"video")
        .await
        .unwrap();
    store.finalize("post", None).await.unwrap();

    assert_eq!(store.continuation_for(&transfer).await.unwrap(), None);
    assert!(store.select_transfer(transfer.clone()).await.is_err());
    assert!(store.select_transfer(mirror).await.is_err());
    assert!(store
        .accept_generation(&transfer, generation("\"two\""))
        .await
        .is_err());
    assert_eq!(
        store.read_range("post", 0..5).await.unwrap(),
        Some(b"video".to_vec())
    );
    assert!(store.is_complete("post").await.unwrap());
    assert!(!root.join("post.generation.json").exists());
    drop(store);

    let reopened = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.unwrap();
    reopened.bind_representation(binding).await.unwrap();
    assert_eq!(reopened.continuation_for(&transfer).await.unwrap(), None);
    assert!(!root.join("post.generation.json").exists());
    assert_eq!(
        reopened.read_range("post", 0..5).await.unwrap(),
        Some(b"video".to_vec())
    );
    store_fixture::discard(&root);
}

fn generation(etag: &str) -> SourceGeneration {
    SourceGeneration::try_new("https://cdn.example/video", etag, 5).unwrap()
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec![
            "https://cdn.example/video".to_owned(),
            "https://mirror.example/video".to_owned(),
        ],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(5),
        duration_ms: Some(1_000),
    }
}
