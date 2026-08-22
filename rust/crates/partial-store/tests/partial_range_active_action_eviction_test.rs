mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{ByteRange, DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn policy_eviction_never_rewrites_beneath_an_active_sparse_response() {
    let fixture =
        store_fixture::spaced_store("active-action-eviction", store_fixture::limits(12, 0), 12);
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta());
    let identity = binding.transfer("https://cdn.example/clip").unwrap();
    let generation = SourceGeneration::try_new(identity.source().as_str(), "\"g\"", 8).unwrap();
    fixture.store.bind_representation(binding).await.unwrap();

    let stable = fixture.store.reserve_action(&identity, 1, 4).await.unwrap();
    fixture
        .store
        .open_sparse_response(&identity, &stable, generation.clone(), ByteRange::new(0, 4))
        .await
        .unwrap();
    fixture
        .store
        .write_range_for_action_if_current(&identity, &generation, &stable, 0, b"abcd")
        .await
        .unwrap();
    fixture
        .store
        .finish_sparse_response(&identity, &generation, &stable)
        .await
        .unwrap();
    fixture.store.release_action(&stable).await;

    let active = fixture.store.reserve_action(&identity, 2, 4).await.unwrap();
    fixture
        .store
        .open_sparse_response(&identity, &active, generation.clone(), ByteRange::new(4, 8))
        .await
        .unwrap();
    fixture
        .store
        .write_range_for_action_if_current(&identity, &generation, &active, 4, b"efgh")
        .await
        .unwrap();

    let eviction = 0..4;
    assert_eq!(
        fixture
            .store
            .evict_ranges("clip", std::slice::from_ref(&eviction))
            .await
            .unwrap()
            .freed_bytes(),
        0
    );
    assert_eq!(
        fixture.store.read_range("clip", 0..8).await.unwrap(),
        Some(b"abcdefgh".to_vec())
    );
    fixture.store.release_action(&active).await;
    store_fixture::discard(&fixture.root);
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
