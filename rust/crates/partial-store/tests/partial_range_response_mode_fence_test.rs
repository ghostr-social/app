mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn resumable_takeover_fences_the_older_single_response() {
    let (root, store, transfer) = fixture("resumable-takeover").await;
    store
        .begin_single_response(&transfer, 1, store_fixture::exact_response(8))
        .await
        .unwrap();
    let generation = generation();
    store
        .accept_generation(&transfer, generation.clone())
        .await
        .unwrap();

    assert!(!store
        .write_single_response_if_current(&transfer, 1, 0, b"stale")
        .await
        .unwrap());
    assert!(store
        .write_range_for_generation_if_current(&transfer, &generation, 0, b"fresh")
        .await
        .unwrap());
    assert_eq!(
        store.read_range("post", 0..5).await.unwrap(),
        Some(b"fresh".to_vec())
    );
    store_fixture::discard(&root);
}

#[tokio::test]
async fn staged_single_response_preserves_sparse_bytes_until_atomic_takeover() {
    let (root, store, transfer) = fixture("single-takeover").await;
    let generation = generation();
    store
        .accept_generation(&transfer, generation.clone())
        .await
        .unwrap();
    store
        .begin_single_response(&transfer, 2, store_fixture::exact_response(8))
        .await
        .unwrap();

    assert!(store
        .write_range_for_generation_if_current(&transfer, &generation, 0, b"old data")
        .await
        .unwrap());
    assert!(store
        .write_single_response_if_current(&transfer, 2, 0, b"new data")
        .await
        .unwrap());
    assert_eq!(
        store.read_range("post", 0..8).await.unwrap(),
        Some(b"old data".to_vec())
    );
    assert!(store
        .finish_single_response(&transfer, 2, Some(8), true)
        .await
        .unwrap());
    assert_eq!(
        store.read_range("post", 0..8).await.unwrap(),
        Some(b"new data".to_vec())
    );
    assert!(!store
        .write_range_for_generation_if_current(&transfer, &generation, 0, b"late")
        .await
        .unwrap());
    store_fixture::discard(&root);
}

async fn fixture(
    prefix: &str,
) -> (
    std::path::PathBuf,
    ghostr_partial_store::partial_range_store::PartialRangeStore,
    ghostr_engine::representation::TransferIdentity,
) {
    let root = store_fixture::temp_root(prefix);
    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let transfer = binding.transfer("https://cdn.example/video").unwrap();
    store.bind_representation(binding).await.unwrap();
    store.select_transfer(transfer.clone()).await.unwrap();
    (root, store, transfer)
}

fn generation() -> SourceGeneration {
    SourceGeneration::try_new("https://cdn.example/video", "\"generation\"", 8).unwrap()
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
