use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use sha2::{Digest as _, Sha256};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn safe_policy_binding_preserves_the_resumable_generation() {
    let root = crate::tests::store_fixture::temp_root("policy-generation-restore");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta());
    let transfer = binding
        .transfer(&meta().urls[0])
        .expect("valid test fixture");
    let generation = SourceGeneration::try_new(&meta().urls[0], "\"generation\"", 16)
        .expect("valid test fixture");
    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    store
        .bind_representation(binding.clone())
        .await
        .expect("valid test fixture");
    store
        .select_transfer(transfer.clone())
        .await
        .expect("valid test fixture");
    store
        .accept_generation(&transfer, generation.clone())
        .await
        .expect("valid test fixture");
    store
        .write_range_for_generation_if_current(&transfer, &generation, 0, b"abcdefgh")
        .await
        .expect("valid test fixture");
    let manifest = tokio::fs::read(root.join("clip.ranges.json"))
        .await
        .expect("valid test fixture");
    let old_hash = format!("{:x}", Sha256::digest(&manifest));
    let intent = format!(
        r#"{{"version":2,"old_accounted":8,"new_accounted":4,"old_manifest_sha256":"{old_hash}"}}"#
    );
    tokio::fs::write(root.join("clip.evict.intent"), intent)
        .await
        .expect("valid test fixture");
    tokio::fs::create_dir(root.join("clip.part.evict"))
        .await
        .expect("valid test fixture");
    drop(store);

    let reopened = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.expect("valid test fixture");
    reopened
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    assert!(root.join("clip.http-generation.json").exists());
    tokio::fs::remove_dir(root.join("clip.part.evict"))
        .await
        .expect("valid test fixture");

    assert_eq!(
        reopened
            .continuation_for(&transfer)
            .await
            .expect("valid test fixture"),
        Some(generation.clone())
    );
    assert_eq!(
        reopened
            .select_transfer(transfer)
            .await
            .expect("valid test fixture"),
        Some(generation)
    );
    assert_eq!(
        reopened
            .read_range("clip", 0..8)
            .await
            .expect("valid test fixture"),
        Some(b"abcdefgh".to_vec())
    );
    crate::tests::store_fixture::discard(&root);
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://media.example/video.mp4".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(16),
        duration_ms: Some(1_000),
    }
}
