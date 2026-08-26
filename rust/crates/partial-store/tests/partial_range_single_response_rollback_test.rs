use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn cancelled_replacement_keeps_the_previous_generation_readable() {
    let root = crate::tests::store_fixture::temp_root("single-response-rollback");
    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let transfer = binding
        .transfer("https://cdn.example/video")
        .expect("valid test fixture");
    let generation = SourceGeneration::try_new("https://cdn.example/video", "\"old\"", 8)
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
        .accept_generation(&transfer, generation.clone())
        .await
        .expect("valid test fixture");
    store
        .write_range_for_generation_if_current(&transfer, &generation, 0, b"old!")
        .await
        .expect("valid test fixture");

    store
        .begin_single_response(&transfer, 3, crate::tests::store_fixture::exact_response(8))
        .await
        .expect("valid test fixture");
    store
        .write_single_response_if_current(&transfer, 3, 0, b"new!")
        .await
        .expect("valid test fixture");
    assert_eq!(
        store
            .read_range("post", 0..4)
            .await
            .expect("valid test fixture"),
        Some(b"old!".to_vec())
    );
    assert!(!store
        .finish_single_response(&transfer, 3, Some(8), false)
        .await
        .expect("valid test fixture"));
    assert!(store
        .write_range_for_generation_if_current(&transfer, &generation, 4, b"keep")
        .await
        .expect("valid test fixture"));
    assert_eq!(
        store
            .read_range("post", 0..8)
            .await
            .expect("valid test fixture"),
        Some(b"old!keep".to_vec())
    );
    crate::tests::store_fixture::discard(&root);
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://cdn.example/video".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
