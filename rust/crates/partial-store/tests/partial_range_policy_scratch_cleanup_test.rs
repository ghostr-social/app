#![cfg(unix)]

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::os::unix::fs::PermissionsExt as _;

#[tokio::test]
async fn failed_policy_scratch_cleanup_preserves_and_accounts_canonical() {
    let fixture = crate::tests::store_fixture::spaced_store(
        "policy-scratch-cleanup",
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
    tokio::fs::write(fixture.root.join("clip.part.evict"), b"abcd0000ijkl")
        .await
        .expect("fixture");
    tokio::fs::write(
        fixture.root.join("clip.evict.intent"),
        br#"{"version":1,"retained_bytes":8}"#,
    )
    .await
    .expect("fixture");
    std::fs::set_permissions(&fixture.root, std::fs::Permissions::from_mode(0o500))
        .expect("fixture");

    let reopened = crate::tests::store_fixture::reopened(&fixture);
    reopened.store.load_existing().await.expect("fixture");
    reopened
        .store
        .bind_representation(binding)
        .await
        .expect("fixture");

    assert_eq!(reopened.store.used_bytes().await, 20);
    assert_eq!(
        reopened
            .store
            .read_range("clip", 4..8)
            .await
            .expect("fixture"),
        Some(b"efgh".to_vec())
    );
    assert!(reopened
        .store
        .reserve_action(&identity, 1, 8)
        .await
        .is_err());
    std::fs::set_permissions(&fixture.root, std::fs::Permissions::from_mode(0o700))
        .expect("fixture");
    let retry = reopened
        .store
        .reserve_action(&identity, 2, 8)
        .await
        .expect("fixture");
    assert_eq!(reopened.store.used_bytes().await, 12);
    assert!(!fixture.root.join("clip.part.evict").exists());
    reopened.store.release_action(&retry).await;
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
