use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn policy_cleanup_charges_only_payload_that_remains_after_a_partial_cleanup() {
    let fixture = crate::tests::store_fixture::spaced_store(
        "policy-partial-cleanup",
        crate::tests::store_fixture::limits(20, 0),
        20,
    );
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta());
    let identity = binding
        .transfer("https://cdn.example/video")
        .expect("fixture");
    fixture
        .store
        .bind_representation(binding.clone())
        .await
        .expect("fixture");
    crate::tests::store_fixture::authorize(&fixture.store, &identity, "generation").await;
    fixture
        .store
        .write_range("clip", 0, b"abcdefghijkl")
        .await
        .expect("fixture");
    fixture
        .store
        .set_total_len("clip", 12)
        .await
        .expect("fixture");
    tokio::fs::write(fixture.root.join("clip.part.evict"), b"abcdefgh")
        .await
        .expect("fixture");
    tokio::fs::create_dir(fixture.root.join("clip.evict.intent"))
        .await
        .expect("fixture");

    let reopened = crate::tests::store_fixture::reopened(&fixture);
    reopened.store.load_existing().await.expect("fixture");
    reopened
        .store
        .bind_representation(binding)
        .await
        .expect("fixture");

    assert!(fixture.root.join("clip.part.evict").exists());
    assert_eq!(reopened.store.used_bytes().await, 20);
    assert!(reopened
        .store
        .reserve_action(&identity, 1, 8)
        .await
        .is_err());
    tokio::fs::remove_dir(fixture.root.join("clip.evict.intent"))
        .await
        .expect("fixture");
    let action = reopened
        .store
        .reserve_action(&identity, 2, 8)
        .await
        .expect("fixture");
    assert!(!fixture.root.join("clip.part.evict").exists());
    assert_eq!(reopened.store.used_bytes().await, 12);
    reopened.store.release_action(&action).await;
    crate::tests::store_fixture::discard(&fixture.root);
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://cdn.example/video".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(12),
        duration_ms: Some(1_000),
    }
}
