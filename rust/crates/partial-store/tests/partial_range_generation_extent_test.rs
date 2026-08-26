use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn coherent_generation_publishes_its_extent_before_body_completion() {
    let root = crate::tests::store_fixture::temp_root("partial-generation-extent");
    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let transfer = binding
        .transfer(&meta().urls[0])
        .expect("valid test fixture");
    let generation = SourceGeneration::try_new(&meta().urls[0], "\"body-generation\"", 16)
        .expect("valid test fixture");
    store
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    store
        .select_transfer(transfer.clone())
        .await
        .expect("valid test fixture");

    store
        .accept_generation(&transfer, generation)
        .await
        .expect("valid test fixture");

    assert_eq!(
        store.total_len("post").await.expect("valid test fixture"),
        Some(16)
    );
    assert!(store
        .present_ranges("post")
        .await
        .expect("valid test fixture")
        .is_empty());
    assert!(!store.is_complete("post").await.expect("valid test fixture"));
    crate::tests::store_fixture::discard(&root);
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://media.example/video.mp4".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
