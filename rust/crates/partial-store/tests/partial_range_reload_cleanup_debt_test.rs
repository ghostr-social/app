#![cfg(unix)]

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::os::unix::fs::PermissionsExt as _;

#[tokio::test]
async fn failed_restart_cleanup_keeps_orphan_bytes_inside_the_hard_cap() {
    let fixture = crate::tests::store_fixture::spaced_store(
        "reload-cleanup-debt",
        crate::tests::store_fixture::limits(4, 0),
        4,
    );
    std::fs::create_dir_all(&fixture.root).expect("valid test fixture");
    std::fs::write(fixture.root.join("post.part"), b"torn").expect("valid test fixture");
    std::fs::set_permissions(&fixture.root, std::fs::Permissions::from_mode(0o500))
        .expect("valid test fixture");

    fixture
        .store
        .load_existing()
        .await
        .expect("valid test fixture");

    assert_eq!(fixture.store.used_bytes().await, 4);
    std::fs::set_permissions(&fixture.root, std::fs::Permissions::from_mode(0o700))
        .expect("valid test fixture");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let identity = binding
        .transfer("https://cdn.example/video")
        .expect("valid test fixture");
    fixture
        .store
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    let action = fixture
        .store
        .reserve_action(&identity, 1, 4)
        .await
        .expect("valid test fixture");
    assert_eq!(fixture.store.used_bytes().await, 0);
    fixture.store.release_action(&action).await;
    crate::tests::store_fixture::discard(&fixture.root);
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://cdn.example/video".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(4),
        duration_ms: Some(1_000),
    }
}
