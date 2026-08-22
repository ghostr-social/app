mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{ByteRange, DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn sparse_response_writes_stay_contiguous_inside_the_returned_range() {
    let fixture = store_fixture::spaced_store("sparse-envelope", store_fixture::limits(8, 0), 8);
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta());
    let identity = binding.transfer("https://cdn.example/clip").unwrap();
    let generation = SourceGeneration::try_new(identity.source().as_str(), "\"g\"", 8).unwrap();
    fixture.store.bind_representation(binding).await.unwrap();
    let action = fixture.store.reserve_action(&identity, 1, 4).await.unwrap();
    fixture
        .store
        .open_sparse_response(&identity, &action, generation.clone(), ByteRange::new(2, 6))
        .await
        .unwrap();

    fixture
        .store
        .write_range_for_action_if_current(&identity, &generation, &action, 3, b"x")
        .await
        .expect_err("response cannot skip its first returned byte");
    fixture
        .store
        .write_range_for_action_if_current(&identity, &generation, &action, 2, b"abcde")
        .await
        .expect_err("response cannot exceed its returned range");
    assert!(fixture
        .store
        .present_ranges("clip")
        .await
        .unwrap()
        .is_empty());
    fixture.store.release_action(&action).await;
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
