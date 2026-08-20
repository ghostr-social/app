mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::ByteRange;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn torn_sparse_write_remains_charged_until_cleanup_succeeds() {
    let fixture =
        store_fixture::spaced_store("torn-sparse-write", store_fixture::limits(16, 0), 16);
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta("post"));
    let identity = binding.transfer("https://cdn.example/post").unwrap();
    let generation = SourceGeneration::try_new(identity.source().as_str(), "\"v1\"", 8).unwrap();
    fixture.store.bind_representation(binding).await.unwrap();
    let action = fixture.store.reserve_action(&identity, 1, 8).await.unwrap();
    fixture
        .store
        .open_sparse_response(&identity, &action, generation.clone(), ByteRange::new(0, 8))
        .await
        .unwrap();
    std::fs::remove_file(fixture.root.join("post.ranges.json")).unwrap();
    std::fs::create_dir(fixture.root.join("post.ranges.json")).unwrap();

    assert!(fixture
        .store
        .write_range_for_action_if_current(&identity, &generation, &action, 0, b"half")
        .await
        .unwrap());
    fixture.store.release_action(&action).await;

    assert_eq!(fixture.store.read_range("post", 0..4).await.unwrap(), None);
    assert_eq!(*fixture.used_bytes.lock().await, 4);
    std::fs::remove_dir(fixture.root.join("post.ranges.json")).unwrap();

    let retry = fixture.store.reserve_action(&identity, 3, 8).await.unwrap();
    assert_eq!(*fixture.used_bytes.lock().await, 0);
    assert!(!fixture.root.join("post.part").exists());
    assert_eq!(
        fixture
            .store
            .open_sparse_response(&identity, &retry, generation, ByteRange::new(0, 8),)
            .await
            .unwrap(),
        ghostr_partial_store::partial_range_store::ResponseOpenResult::Opened
    );
    fixture.store.release_action(&retry).await;
    store_fixture::discard(&fixture.root);
}

fn meta(name: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![format!("https://cdn.example/{name}")],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
