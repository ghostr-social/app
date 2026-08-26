#![cfg(unix)]

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::os::unix::fs::PermissionsExt as _;

#[tokio::test]
async fn policy_debt_retry_never_removes_a_manifest_writers_staging_file() {
    let fixture = crate::tests::store_fixture::spaced_store(
        "policy-cleanup-scope",
        crate::tests::store_fixture::limits(32, 0),
        32,
    );
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta());
    let identity = binding
        .transfer("https://cdn.example/video")
        .expect("valid test fixture");
    fixture
        .store
        .bind_representation(binding.clone())
        .await
        .expect("valid test fixture");
    fixture
        .store
        .write_range("clip", 0, b"abcdefghijkl")
        .await
        .expect("valid test fixture");
    fixture
        .store
        .set_total_len("clip", 12)
        .await
        .expect("valid test fixture");
    tokio::fs::write(fixture.root.join("clip.part.evict"), b"abcdefgh")
        .await
        .expect("valid test fixture");
    tokio::fs::write(
        fixture.root.join("clip.evict.intent"),
        br#"{"version":1,"retained_bytes":8}"#,
    )
    .await
    .expect("valid test fixture");
    std::fs::set_permissions(&fixture.root, std::fs::Permissions::from_mode(0o500))
        .expect("valid test fixture");
    let reopened = crate::tests::store_fixture::reopened(&fixture);
    reopened
        .store
        .load_existing()
        .await
        .expect("valid test fixture");

    std::fs::set_permissions(&fixture.root, std::fs::Permissions::from_mode(0o700))
        .expect("valid test fixture");
    let writer_staging = fixture.root.join("clip.ranges.json.tmp");
    tokio::fs::write(&writer_staging, b"writer-owned")
        .await
        .expect("valid test fixture");
    reopened
        .store
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    let action = reopened
        .store
        .reserve_action(&identity, 1, 8)
        .await
        .expect("valid test fixture");

    assert_eq!(
        tokio::fs::read(writer_staging)
            .await
            .expect("valid test fixture"),
        b"writer-owned"
    );
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
