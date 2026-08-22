#![cfg(unix)]

mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::os::unix::fs::PermissionsExt;

#[tokio::test]
async fn policy_debt_retry_never_removes_a_manifest_writers_staging_file() {
    let fixture =
        store_fixture::spaced_store("policy-cleanup-scope", store_fixture::limits(32, 0), 32);
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
    let writer_staging = fixture.root.join("clip.ranges.json.tmp");
    tokio::fs::write(&writer_staging, b"writer-owned")
        .await
        .unwrap();
    reopened.store.bind_representation(binding).await.unwrap();
    let action = reopened
        .store
        .reserve_action(&identity, 1, 8)
        .await
        .unwrap();

    assert_eq!(
        tokio::fs::read(writer_staging).await.unwrap(),
        b"writer-owned"
    );
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
