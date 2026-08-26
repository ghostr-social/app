use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn short_live_response_is_discarded_before_a_retry() {
    let root = crate::tests::store_fixture::temp_root("single-response-short");
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
    assert!(store
        .begin_single_response(&transfer, 1, crate::tests::store_fixture::exact_response(8))
        .await
        .expect("valid test fixture"));
    store
        .write_single_response_if_current(&transfer, 1, 0, b"short")
        .await
        .expect("valid test fixture");

    store
        .finish_single_response(&transfer, 1, Some(8), true)
        .await
        .expect_err("framed response ended before the declared length");

    assert_eq!(
        store
            .read_range("post", 0..5)
            .await
            .expect("valid test fixture"),
        None
    );
    assert!(!store
        .begin_single_response(&transfer, 2, crate::tests::store_fixture::exact_response(8))
        .await
        .expect("valid test fixture"));
    store
        .select_transfer(transfer.clone())
        .await
        .expect("valid test fixture");
    assert!(store
        .begin_single_response(&transfer, 2, crate::tests::store_fixture::exact_response(8))
        .await
        .expect("valid test fixture"));
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
