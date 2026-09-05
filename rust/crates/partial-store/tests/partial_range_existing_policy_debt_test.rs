#![cfg(unix)]

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::os::unix::fs::PermissionsExt as _;

#[tokio::test]
async fn a_new_policy_rewrite_reconciles_an_older_cleanup_debt_first() {
    let fixture = crate::tests::store_fixture::spaced_store(
        "existing-policy-debt",
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
    std::fs::set_permissions(&fixture.root, std::fs::Permissions::from_mode(0o700))
        .expect("fixture");
    reopened
        .store
        .bind_representation(binding)
        .await
        .expect("fixture");
    let manifest = fixture.root.join("clip.ranges.json");
    let stable = tokio::fs::read(&manifest).await.expect("fixture");
    tokio::fs::remove_file(&manifest).await.expect("fixture");
    tokio::fs::create_dir(&manifest).await.expect("fixture");

    reopened
        .store
        .evict_ranges("clip", core::slice::from_ref(&(4..8)))
        .await
        .expect_err("scenario must fail");

    assert_eq!(reopened.store.used_bytes().await, 12);
    assert!(!fixture.root.join("clip.part.evict").exists());
    tokio::fs::remove_dir(&manifest).await.expect("fixture");
    tokio::fs::write(manifest, stable).await.expect("fixture");
    let action = reopened
        .store
        .reserve_action(&identity, 1, 8)
        .await
        .expect("fixture");
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
