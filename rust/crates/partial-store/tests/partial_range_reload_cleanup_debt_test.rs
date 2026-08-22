#![cfg(unix)]

mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::os::unix::fs::PermissionsExt;

#[tokio::test]
async fn failed_restart_cleanup_keeps_orphan_bytes_inside_the_hard_cap() {
    let fixture =
        store_fixture::spaced_store("reload-cleanup-debt", store_fixture::limits(4, 0), 4);
    std::fs::create_dir_all(&fixture.root).unwrap();
    std::fs::write(fixture.root.join("post.part"), b"torn").unwrap();
    std::fs::set_permissions(&fixture.root, std::fs::Permissions::from_mode(0o500)).unwrap();

    fixture.store.load_existing().await.unwrap();

    assert_eq!(fixture.store.used_bytes().await, 4);
    std::fs::set_permissions(&fixture.root, std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let identity = binding.transfer("https://cdn.example/video").unwrap();
    fixture.store.bind_representation(binding).await.unwrap();
    let action = fixture.store.reserve_action(&identity, 1, 4).await.unwrap();
    assert_eq!(fixture.store.used_bytes().await, 0);
    fixture.store.release_action(&action).await;
    store_fixture::discard(&fixture.root);
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
