use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{ByteRange, DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn policy_eviction_never_rewrites_beneath_an_active_sparse_response() {
    let fixture = crate::tests::store_fixture::spaced_store(
        "active-action-eviction",
        crate::tests::store_fixture::limits(12, 0),
        12,
    );
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta());
    let identity = binding
        .transfer("https://cdn.example/clip")
        .expect("valid test fixture");
    let generation = SourceGeneration::try_new(identity.source().as_str(), "\"g\"", 8)
        .expect("valid test fixture");
    fixture
        .store
        .bind_representation(binding)
        .await
        .expect("valid test fixture");

    let stable = fixture
        .store
        .reserve_action(&identity, 1, 4)
        .await
        .expect("valid test fixture");
    fixture
        .store
        .open_sparse_response(&identity, &stable, generation.clone(), ByteRange::new(0, 4))
        .await
        .expect("valid test fixture");
    fixture
        .store
        .write_range_for_action_if_current(&identity, &generation, &stable, 0, b"abcd")
        .await
        .expect("valid test fixture");
    fixture
        .store
        .finish_sparse_response(&identity, &generation, &stable)
        .await
        .expect("valid test fixture");
    fixture.store.release_action(&stable).await;

    let active = fixture
        .store
        .reserve_action(&identity, 2, 4)
        .await
        .expect("valid test fixture");
    fixture
        .store
        .open_sparse_response(&identity, &active, generation.clone(), ByteRange::new(4, 8))
        .await
        .expect("valid test fixture");
    fixture
        .store
        .write_range_for_action_if_current(&identity, &generation, &active, 4, b"efgh")
        .await
        .expect("valid test fixture");

    let eviction = 0..4;
    assert_eq!(
        fixture
            .store
            .evict_ranges("clip", core::slice::from_ref(&eviction))
            .await
            .expect("valid test fixture")
            .freed_bytes(),
        0
    );
    assert_eq!(
        fixture
            .store
            .read_range("clip", 0..8)
            .await
            .expect("valid test fixture"),
        Some(b"abcdefgh".to_vec())
    );
    fixture.store.release_action(&active).await;
    crate::tests::store_fixture::discard(&fixture.root);
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://cdn.example/clip".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
