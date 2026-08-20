mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn full_eviction_removes_the_generation_that_owned_the_bytes() {
    let fixture = store_fixture::spaced_store(
        "partial-eviction-provenance",
        store_fixture::limits(16, 0),
        1_000,
    );
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let transfer = binding.transfer(&meta().urls[0]).unwrap();
    let generation = SourceGeneration::try_new(&meta().urls[0], "\"one\"", 8).unwrap();
    fixture.store.bind_representation(binding).await.unwrap();
    fixture
        .store
        .select_transfer(transfer.clone())
        .await
        .unwrap();
    fixture
        .store
        .accept_generation(&transfer, generation.clone())
        .await
        .unwrap();
    fixture
        .store
        .write_range_for_generation_if_current(&transfer, &generation, 0, b"old data")
        .await
        .unwrap();
    fixture
        .store
        .write_range("newer", 0, b"new data")
        .await
        .unwrap();

    fixture.store.set_storage_budget(8).await.unwrap();

    let snapshot = fixture.store.media_snapshot("post").await.unwrap();
    assert!(snapshot.ranges().is_empty());
    assert_eq!(snapshot.continuation_source(), None);
    assert!(!fixture
        .store
        .write_range_for_generation_if_current(&transfer, &generation, 0, b"stale")
        .await
        .unwrap());
    assert_eq!(fixture.store.select_transfer(transfer).await.unwrap(), None);
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
