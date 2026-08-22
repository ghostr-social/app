#![cfg(unix)]

mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::os::unix::fs::PermissionsExt;

#[tokio::test]
async fn failed_stage_cleanup_on_restart_charges_stage_and_preserves_canonical() {
    let fixture =
        store_fixture::spaced_store("staged-cleanup-reload", store_fixture::limits(8, 0), 8);
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let primary = binding.transfer("https://a.example/video").unwrap();
    let mirror = binding.transfer("https://b.example/video").unwrap();
    let generation = SourceGeneration::try_new(primary.source().as_str(), "\"a\"", 8).unwrap();
    fixture
        .store
        .bind_representation(binding.clone())
        .await
        .unwrap();
    fixture
        .store
        .select_transfer(primary.clone())
        .await
        .unwrap();
    fixture
        .store
        .accept_generation(&primary, generation.clone())
        .await
        .unwrap();
    fixture
        .store
        .write_range_for_generation_if_current(&primary, &generation, 0, b"old!")
        .await
        .unwrap();
    let action = fixture.store.reserve_action(&mirror, 1, 4).await.unwrap();
    fixture
        .store
        .open_single_response_for_action(&mirror, &action, store_fixture::exact_response(4))
        .await
        .unwrap();
    fixture
        .store
        .write_single_response_for_action(&mirror, &action, 0, b"new!")
        .await
        .unwrap();
    drop(action);
    std::fs::set_permissions(&fixture.root, std::fs::Permissions::from_mode(0o500)).unwrap();

    let reopened = store_fixture::reopened(&fixture);
    reopened.store.load_existing().await.unwrap();

    assert_eq!(reopened.store.used_bytes().await, 8);
    assert_eq!(
        reopened.store.read_range("post", 0..4).await.unwrap(),
        Some(b"old!".to_vec())
    );
    assert!(fixture.root.join("post.response.part").exists());
    std::fs::set_permissions(&fixture.root, std::fs::Permissions::from_mode(0o700)).unwrap();
    reopened.store.bind_representation(binding).await.unwrap();
    let retry = reopened.store.reserve_action(&mirror, 2, 4).await.unwrap();
    assert_eq!(reopened.store.used_bytes().await, 4);
    assert!(!fixture.root.join("post.response.part").exists());
    assert_eq!(
        reopened.store.read_range("post", 0..4).await.unwrap(),
        Some(b"old!".to_vec())
    );
    reopened.store.release_action(&retry).await;
    store_fixture::discard(&fixture.root);
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec![
            "https://a.example/video".to_owned(),
            "https://b.example/video".to_owned(),
        ],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
