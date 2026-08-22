#![cfg(unix)]

mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::os::unix::fs::PermissionsExt;

#[tokio::test]
async fn a_new_policy_rewrite_reconciles_an_older_cleanup_debt_first() {
    let fixture =
        store_fixture::spaced_store("existing-policy-debt", store_fixture::limits(20, 0), 20);
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
    tokio::fs::write(
        fixture.root.join("clip.evict.intent"),
        br#"{"version":1,"retained_bytes":8}"#,
    )
    .await
    .unwrap();
    std::fs::set_permissions(&fixture.root, std::fs::Permissions::from_mode(0o500)).unwrap();
    let reopened = store_fixture::reopened(&fixture);
    reopened.store.load_existing().await.unwrap();
    std::fs::set_permissions(&fixture.root, std::fs::Permissions::from_mode(0o700)).unwrap();
    reopened.store.bind_representation(binding).await.unwrap();
    let manifest = fixture.root.join("clip.ranges.json");
    let stable = tokio::fs::read(&manifest).await.unwrap();
    tokio::fs::remove_file(&manifest).await.unwrap();
    tokio::fs::create_dir(&manifest).await.unwrap();

    reopened
        .store
        .evict_ranges("clip", std::slice::from_ref(&(4..8)))
        .await
        .unwrap_err();

    assert_eq!(reopened.store.used_bytes().await, 12);
    assert!(!fixture.root.join("clip.part.evict").exists());
    tokio::fs::remove_dir(&manifest).await.unwrap();
    tokio::fs::write(manifest, stable).await.unwrap();
    let action = reopened
        .store
        .reserve_action(&identity, 1, 8)
        .await
        .unwrap();
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
