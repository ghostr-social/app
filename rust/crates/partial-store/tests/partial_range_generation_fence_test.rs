mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn changed_validator_replaces_bytes_and_fences_late_writes() {
    let root = store_fixture::temp_root("partial-generation-fence");
    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("same"), meta());
    let transfer = binding.transfer("https://a.example/video").unwrap();
    let first = generation("https://cdn.example/one", "\"one\"");
    let second = generation("https://cdn.example/two", "\"two\"");
    store.bind_representation(binding).await.unwrap();
    store.select_transfer(transfer.clone()).await.unwrap();
    store
        .accept_generation(&transfer, first.clone())
        .await
        .unwrap();
    assert!(store
        .write_range_for_generation_if_current(&transfer, &first, 0, b"old!")
        .await
        .unwrap());

    store
        .accept_generation(&transfer, second.clone())
        .await
        .unwrap();

    assert!(!store
        .write_range_for_generation_if_current(&transfer, &first, 4, b"late")
        .await
        .unwrap());
    assert!(store
        .write_range_for_generation_if_current(&transfer, &second, 4, b"new!")
        .await
        .unwrap());
    assert_eq!(store.read_range("same", 0..4).await.unwrap(), None);
    store_fixture::discard(&root);
}

fn generation(url: &str, etag: &str) -> SourceGeneration {
    SourceGeneration::try_new(url, etag, 8).unwrap()
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
