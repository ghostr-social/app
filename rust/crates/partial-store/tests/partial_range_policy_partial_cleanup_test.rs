mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn policy_cleanup_charges_only_payload_that_remains_after_a_partial_cleanup() {
    let fixture =
        store_fixture::spaced_store("policy-partial-cleanup", store_fixture::limits(20, 0), 20);
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta());
    let identity = binding.transfer("https://cdn.example/video").unwrap();
    fixture
        .store
        .bind_representation(binding.clone())
        .await
        .unwrap();
    fixture
        .store
        .write_range("clip", 0, b"abcdefghijkl")
        .await
        .unwrap();
    fixture.store.set_total_len("clip", 12).await.unwrap();
    tokio::fs::write(fixture.root.join("clip.part.evict"), b"abcdefgh")
        .await
        .unwrap();
    tokio::fs::create_dir(fixture.root.join("clip.evict.intent"))
        .await
        .unwrap();

    let reopened = store_fixture::reopened(&fixture);
    reopened.store.load_existing().await.unwrap();
    reopened.store.bind_representation(binding).await.unwrap();

    assert!(fixture.root.join("clip.part.evict").exists());
    assert_eq!(reopened.store.used_bytes().await, 20);
    assert!(reopened
        .store
        .reserve_action(&identity, 1, 8)
        .await
        .is_err());
    tokio::fs::remove_dir(fixture.root.join("clip.evict.intent"))
        .await
        .unwrap();
    let action = reopened
        .store
        .reserve_action(&identity, 2, 8)
        .await
        .unwrap();
    assert!(!fixture.root.join("clip.part.evict").exists());
    assert_eq!(reopened.store.used_bytes().await, 12);
    reopened.store.release_action(&action).await;
    store_fixture::discard(&fixture.root);
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
