use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::ByteRange;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn torn_sparse_write_remains_charged_until_cleanup_succeeds() {
    let fixture = crate::tests::store_fixture::spaced_store(
        "torn-sparse-write",
        crate::tests::store_fixture::limits(16, 0),
        16,
    );
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta("post"));
    let identity = binding
        .transfer("https://cdn.example/post")
        .expect("valid test fixture");
    let generation = SourceGeneration::try_new(identity.source().as_str(), "\"v1\"", 8)
        .expect("valid test fixture");
    fixture
        .store
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    let action = fixture
        .store
        .reserve_action(&identity, 1, 8)
        .await
        .expect("valid test fixture");
    fixture
        .store
        .open_sparse_response(&identity, &action, generation.clone(), ByteRange::new(0, 8))
        .await
        .expect("valid test fixture");
    std::fs::remove_file(fixture.root.join("post.ranges.json")).expect("valid test fixture");
    std::fs::create_dir(fixture.root.join("post.ranges.json")).expect("valid test fixture");

    assert!(fixture
        .store
        .write_range_for_action_if_current(&identity, &generation, &action, 0, b"half")
        .await
        .expect("valid test fixture"));
    fixture.store.release_action(&action).await;

    assert_eq!(
        fixture
            .store
            .read_range("post", 0..4)
            .await
            .expect("valid test fixture"),
        None
    );
    assert_eq!(*fixture.used_bytes.lock().await, 4);
    std::fs::remove_dir(fixture.root.join("post.ranges.json")).expect("valid test fixture");

    let retry = fixture
        .store
        .reserve_action(&identity, 3, 8)
        .await
        .expect("valid test fixture");
    assert_eq!(*fixture.used_bytes.lock().await, 0);
    assert!(!fixture.root.join("post.part").exists());
    assert_eq!(
        fixture
            .store
            .open_sparse_response(&identity, &retry, generation, ByteRange::new(0, 8),)
            .await
            .expect("valid test fixture"),
        crate::partial_range_store::ResponseOpenResult::Opened
    );
    fixture.store.release_action(&retry).await;
    crate::tests::store_fixture::discard(&fixture.root);
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
