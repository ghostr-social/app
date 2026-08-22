#![cfg(unix)]

mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::os::unix::fs::PermissionsExt;

#[tokio::test]
async fn failed_policy_scratch_cleanup_preserves_and_accounts_canonical() {
    let fixture =
        store_fixture::spaced_store("policy-scratch-cleanup", store_fixture::limits(20, 0), 20);
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
    tokio::fs::write(fixture.root.join("clip.part.evict"), b"abcd0000ijkl")
        .await
        .unwrap();
    tokio::fs::write(
        fixture.root.join("clip.evict.intent"),
        br#"{"version":1,"retained_bytes":8}"#,
    )
    .await
    .unwrap();
    std::fs::set_permissions(&fixture.root, std::fs::Permissions::from_mode(0o500)).unwrap();

    let reopened = store_fixture::reopened(&fixture);
    reopened.store.load_existing().await.unwrap();
    reopened.store.bind_representation(binding).await.unwrap();

    assert_eq!(reopened.store.used_bytes().await, 20);
    assert_eq!(
        reopened.store.read_range("clip", 4..8).await.unwrap(),
        Some(b"efgh".to_vec())
    );
    assert!(reopened
        .store
        .reserve_action(&identity, 1, 8)
        .await
        .is_err());
    std::fs::set_permissions(&fixture.root, std::fs::Permissions::from_mode(0o700)).unwrap();
    let retry = reopened
        .store
        .reserve_action(&identity, 2, 8)
        .await
        .unwrap();
    assert_eq!(reopened.store.used_bytes().await, 12);
    assert!(!fixture.root.join("clip.part.evict").exists());
    reopened.store.release_action(&retry).await;
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
