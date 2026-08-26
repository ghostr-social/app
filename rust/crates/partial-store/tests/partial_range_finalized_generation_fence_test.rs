use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn late_generation_cannot_replace_a_finalized_video() {
    let root = crate::tests::store_fixture::temp_root("partial-finalized-generation");
    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let transfer = binding
        .transfer(&meta().urls[0])
        .expect("valid test fixture");
    let mirror = binding
        .transfer(&meta().urls[1])
        .expect("valid test fixture");
    let first = generation("\"one\"");
    store
        .bind_representation(binding.clone())
        .await
        .expect("valid test fixture");
    store
        .select_transfer(transfer.clone())
        .await
        .expect("valid test fixture");
    store
        .accept_generation(&transfer, first.clone())
        .await
        .expect("valid test fixture");
    store
        .write_range_for_generation_if_current(&transfer, &first, 0, b"video")
        .await
        .expect("valid test fixture");
    store
        .finalize("post", None)
        .await
        .expect("valid test fixture");

    assert_eq!(
        store
            .continuation_for(&transfer)
            .await
            .expect("valid test fixture"),
        None
    );
    assert!(store.select_transfer(transfer.clone()).await.is_err());
    assert!(store.select_transfer(mirror).await.is_err());
    assert!(store
        .accept_generation(&transfer, generation("\"two\""))
        .await
        .is_err());
    assert_eq!(
        store
            .read_range("post", 0..5)
            .await
            .expect("valid test fixture"),
        Some(b"video".to_vec())
    );
    assert!(store.is_complete("post").await.expect("valid test fixture"));
    assert!(!root.join("post.generation.json").exists());
    drop(store);

    let reopened = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.expect("valid test fixture");
    reopened
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    assert_eq!(
        reopened
            .continuation_for(&transfer)
            .await
            .expect("valid test fixture"),
        None
    );
    assert!(!root.join("post.generation.json").exists());
    assert_eq!(
        reopened
            .read_range("post", 0..5)
            .await
            .expect("valid test fixture"),
        Some(b"video".to_vec())
    );
    crate::tests::store_fixture::discard(&root);
}

fn generation(etag: &str) -> SourceGeneration {
    SourceGeneration::try_new("https://cdn.example/video", etag, 5).expect("valid test fixture")
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec![
            "https://cdn.example/video".to_owned(),
            "https://mirror.example/video".to_owned(),
        ],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(5),
        duration_ms: Some(1_000),
    }
}
