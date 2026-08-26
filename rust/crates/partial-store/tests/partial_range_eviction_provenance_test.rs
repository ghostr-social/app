use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn full_eviction_removes_the_generation_that_owned_the_bytes() {
    let fixture = crate::tests::store_fixture::spaced_store(
        "partial-eviction-provenance",
        crate::tests::store_fixture::limits(16, 0),
        1_000,
    );
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let transfer = binding
        .transfer(&meta().urls[0])
        .expect("valid test fixture");
    let generation =
        SourceGeneration::try_new(&meta().urls[0], "\"one\"", 8).expect("valid test fixture");
    fixture
        .store
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    fixture
        .store
        .select_transfer(transfer.clone())
        .await
        .expect("valid test fixture");
    fixture
        .store
        .accept_generation(&transfer, generation.clone())
        .await
        .expect("valid test fixture");
    fixture
        .store
        .write_range_for_generation_if_current(&transfer, &generation, 0, b"old data")
        .await
        .expect("valid test fixture");
    fixture
        .store
        .write_range("newer", 0, b"new data")
        .await
        .expect("valid test fixture");

    fixture
        .store
        .set_storage_budget(8)
        .await
        .expect("valid test fixture");

    let snapshot = fixture
        .store
        .media_snapshot("post")
        .await
        .expect("valid test fixture");
    assert!(snapshot.ranges().is_empty());
    assert_eq!(snapshot.continuation_source(), None);
    assert!(!fixture
        .store
        .write_range_for_generation_if_current(&transfer, &generation, 0, b"stale")
        .await
        .expect("valid test fixture"));
    assert_eq!(
        fixture
            .store
            .select_transfer(transfer)
            .await
            .expect("valid test fixture"),
        None
    );
    crate::tests::store_fixture::discard(&fixture.root);
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
