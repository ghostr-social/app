mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn policy_recovery_never_restores_a_foreign_generation() {
    let root = store_fixture::temp_root("policy-generation-provenance");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta());
    let transfer = binding.transfer(&meta().urls[0]).unwrap();
    let generation = SourceGeneration::try_new(&meta().urls[0], "\"generation\"", 16).unwrap();
    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    store.bind_representation(binding.clone()).await.unwrap();
    store.select_transfer(transfer.clone()).await.unwrap();
    store
        .accept_generation(&transfer, generation.clone())
        .await
        .unwrap();
    store
        .write_range_for_generation_if_current(&transfer, &generation, 0, b"abcdefgh")
        .await
        .unwrap();
    replace_generation_fingerprint(&root).await;
    install_blocked_policy_cleanup(&root).await;
    drop(store);

    let reopened = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.unwrap();
    reopened.bind_representation(binding).await.unwrap();
    tokio::fs::remove_dir(root.join("clip.part.evict"))
        .await
        .unwrap();

    assert_eq!(
        reopened.select_transfer(transfer.clone()).await.unwrap(),
        None
    );
    assert!(!reopened
        .write_range_for_generation_if_current(&transfer, &generation, 8, b"ijklmnop")
        .await
        .unwrap());
    assert_eq!(reopened.read_range("clip", 0..8).await.unwrap(), None);
    store_fixture::discard(&root);
}

async fn replace_generation_fingerprint(root: &std::path::Path) {
    let path = root.join("clip.generation.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&tokio::fs::read(&path).await.unwrap()).unwrap();
    value["representation"] = "foreign-representation".into();
    tokio::fs::write(path, serde_json::to_vec(&value).unwrap())
        .await
        .unwrap();
}

async fn install_blocked_policy_cleanup(root: &std::path::Path) {
    let manifest = tokio::fs::read(root.join("clip.ranges.json"))
        .await
        .unwrap();
    let old_hash = format!("{:x}", Sha256::digest(&manifest));
    let intent = format!(
        r#"{{"version":2,"old_accounted":8,"new_accounted":4,"old_manifest_sha256":"{old_hash}"}}"#
    );
    tokio::fs::write(root.join("clip.evict.intent"), intent)
        .await
        .unwrap();
    tokio::fs::create_dir(root.join("clip.part.evict"))
        .await
        .unwrap();
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
