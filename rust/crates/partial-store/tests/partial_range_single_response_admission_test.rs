use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn single_response_keeps_existing_bytes_until_its_first_write_is_admitted() {
    let root = crate::tests::store_fixture::temp_root("single-response-admission");
    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let transfer = binding
        .transfer("https://cdn.example/video")
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
        .write_range("post", 0, b"kept")
        .await
        .expect("valid test fixture");

    assert!(store
        .begin_single_response(&transfer, 7, crate::tests::store_fixture::exact_response(8))
        .await
        .expect("valid test fixture"));

    assert_eq!(
        store
            .read_range("post", 0..4)
            .await
            .expect("valid test fixture"),
        Some(b"kept".to_vec())
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
