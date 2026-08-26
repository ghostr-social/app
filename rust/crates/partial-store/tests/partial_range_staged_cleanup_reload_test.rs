#![cfg(unix)]

use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::os::unix::fs::PermissionsExt as _;

#[tokio::test]
async fn failed_stage_cleanup_on_restart_charges_stage_and_preserves_canonical() {
    let fixture = crate::tests::store_fixture::spaced_store(
        "staged-cleanup-reload",
        crate::tests::store_fixture::limits(8, 0),
        8,
    );
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let primary = binding
        .transfer("https://a.example/video")
        .expect("valid test fixture");
    let mirror = binding
        .transfer("https://b.example/video")
        .expect("valid test fixture");
    let generation = SourceGeneration::try_new(primary.source().as_str(), "\"a\"", 8)
        .expect("valid test fixture");
    fixture
        .store
        .bind_representation(binding.clone())
        .await
        .expect("valid test fixture");
    fixture
        .store
        .select_transfer(primary.clone())
        .await
        .expect("valid test fixture");
    fixture
        .store
        .accept_generation(&primary, generation.clone())
        .await
        .expect("valid test fixture");
    fixture
        .store
        .write_range_for_generation_if_current(&primary, &generation, 0, b"old!")
        .await
        .expect("valid test fixture");
    let action = fixture
        .store
        .reserve_action(&mirror, 1, 4)
        .await
        .expect("valid test fixture");
    fixture
        .store
        .open_single_response_for_action(
            &mirror,
            &action,
            crate::tests::store_fixture::exact_response(4),
        )
        .await
        .expect("valid test fixture");
    fixture
        .store
        .write_single_response_for_action(&mirror, &action, 0, b"new!")
        .await
        .expect("valid test fixture");
    drop(action);
    std::fs::set_permissions(&fixture.root, std::fs::Permissions::from_mode(0o500))
        .expect("valid test fixture");

    let reopened = crate::tests::store_fixture::reopened(&fixture);
    reopened
        .store
        .load_existing()
        .await
        .expect("valid test fixture");

    assert_eq!(reopened.store.used_bytes().await, 8);
    assert_eq!(
        reopened
            .store
            .read_range("post", 0..4)
            .await
            .expect("valid test fixture"),
        Some(b"old!".to_vec())
    );
    assert!(fixture.root.join("post.response.part").exists());
    std::fs::set_permissions(&fixture.root, std::fs::Permissions::from_mode(0o700))
        .expect("valid test fixture");
    reopened
        .store
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    let retry = reopened
        .store
        .reserve_action(&mirror, 2, 4)
        .await
        .expect("valid test fixture");
    assert_eq!(reopened.store.used_bytes().await, 4);
    assert!(!fixture.root.join("post.response.part").exists());
    assert_eq!(
        reopened
            .store
            .read_range("post", 0..4)
            .await
            .expect("valid test fixture"),
        Some(b"old!".to_vec())
    );
    reopened.store.release_action(&retry).await;
    crate::tests::store_fixture::discard(&fixture.root);
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
