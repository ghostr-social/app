use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
#[path = "partial_range_policy_generation_provenance_test/fixture.rs"]
mod corruption;
use corruption::{install_blocked_policy_cleanup, replace_generation_fingerprint};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn policy_recovery_never_restores_a_foreign_generation() {
    let root = crate::tests::store_fixture::temp_root("policy-generation-provenance");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta());
    let transfer = binding.transfer(&meta().urls[0]).expect("fixture");
    let generation =
        SourceGeneration::try_new(&meta().urls[0], "\"generation\"", 16).expect("fixture");
    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    store
        .bind_representation(binding.clone())
        .await
        .expect("fixture");
    store
        .select_transfer(transfer.clone())
        .await
        .expect("fixture");
    store
        .accept_generation(&transfer, generation.clone())
        .await
        .expect("fixture");
    store
        .write_range_for_generation_if_current(&transfer, &generation, 0, b"abcdefgh")
        .await
        .expect("fixture");
    replace_generation_fingerprint(&root).await;
    install_blocked_policy_cleanup(&root).await;
    drop(store);

    let reopened = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.expect("fixture");
    assert!(
        reopened.bind_representation(binding.clone()).await.is_err(),
        "foreign authority stays unavailable while cleanup is blocked"
    );
    tokio::fs::remove_dir(root.join("clip.part.evict"))
        .await
        .expect("fixture");

    assert!(
        reopened.bind_representation(binding.clone()).await.is_err(),
        "recovery rejects and removes the foreign payload"
    );
    reopened
        .bind_representation(binding)
        .await
        .expect("clean binding can now open");

    assert_eq!(
        reopened
            .select_transfer(transfer.clone())
            .await
            .expect("fixture"),
        None
    );
    assert!(!reopened
        .write_range_for_generation_if_current(&transfer, &generation, 8, b"ijklmnop")
        .await
        .expect("fixture"));
    assert_eq!(
        reopened.read_range("clip", 0..8).await.expect("fixture"),
        None
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
