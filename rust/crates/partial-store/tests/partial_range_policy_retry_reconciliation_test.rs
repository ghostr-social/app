#![cfg(unix)]

mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::os::unix::fs::PermissionsExt;

#[tokio::test]
async fn policy_retry_releases_payload_debt_after_a_partial_cleanup() {
    let fixture =
        store_fixture::spaced_store("policy-retry-reconcile", store_fixture::limits(20, 0), 20);
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
    let intent = fixture.root.join("clip.evict.intent");
    tokio::fs::write(&intent, br#"{"version":1,"retained_bytes":8}"#)
        .await
        .unwrap();
    std::fs::set_permissions(&fixture.root, std::fs::Permissions::from_mode(0o500)).unwrap();
    let reopened = store_fixture::reopened(&fixture);
    reopened.store.load_existing().await.unwrap();
    reopened.store.bind_representation(binding).await.unwrap();
    std::fs::set_permissions(&fixture.root, std::fs::Permissions::from_mode(0o700)).unwrap();
    tokio::fs::remove_file(&intent).await.unwrap();
    tokio::fs::create_dir(&intent).await.unwrap();

    assert!(reopened
        .store
        .reserve_action(&identity, 1, 8)
        .await
        .is_err());
    assert!(fixture.root.join("clip.part.evict").exists());
    assert_eq!(reopened.store.used_bytes().await, 20);
    assert!(reopened
        .store
        .reserve_action(&identity, 2, 8)
        .await
        .is_err());
    tokio::fs::remove_dir(intent).await.unwrap();
    let action = reopened
        .store
        .reserve_action(&identity, 3, 8)
        .await
        .unwrap();
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
