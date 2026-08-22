mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::ByteRange;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn unfinished_sparse_action_is_never_reused_after_restart() {
    let fixture =
        store_fixture::spaced_store("sparse-provisional-restart", store_fixture::limits(8, 0), 8);
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta());
    let identity = binding.transfer("https://cdn.example/clip").unwrap();
    let generation = SourceGeneration::try_new(identity.source().as_str(), "\"g\"", 8).unwrap();
    fixture.store.bind_representation(binding).await.unwrap();
    let stable = fixture.store.reserve_action(&identity, 1, 4).await.unwrap();
    let dirty = fixture.store.reserve_action(&identity, 2, 4).await.unwrap();
    fixture
        .store
        .open_sparse_response(&identity, &stable, generation.clone(), ByteRange::new(0, 4))
        .await
        .unwrap();
    fixture
        .store
        .open_sparse_response(&identity, &dirty, generation.clone(), ByteRange::new(4, 8))
        .await
        .unwrap();
    fixture
        .store
        .write_range_for_action_if_current(&identity, &generation, &stable, 0, b"abcd")
        .await
        .unwrap();
    fixture
        .store
        .write_range_for_action_if_current(&identity, &generation, &dirty, 4, b"efgh")
        .await
        .unwrap();
    fixture
        .store
        .finish_sparse_response(&identity, &generation, &stable)
        .await
        .unwrap();

    let reopened = store_fixture::reopened(&fixture);
    reopened.store.load_existing().await.unwrap();
    assert!(reopened
        .store
        .present_ranges("clip")
        .await
        .unwrap()
        .is_empty());
    assert_eq!(reopened.store.used_bytes().await, 0);
    assert!(!reopened.root.join("clip.part").exists());
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
