mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn beginning_a_response_does_not_rewrite_its_persisted_representation() {
    let root = store_fixture::temp_root("single-response-begin-transaction");
    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let transfer = binding.transfer("https://cdn.example/video").unwrap();
    store.bind_representation(binding).await.unwrap();
    store.select_transfer(transfer.clone()).await.unwrap();
    std::fs::create_dir(root.join("post.representation.tmp")).unwrap();

    assert!(store
        .begin_single_response(&transfer, 1, store_fixture::exact_response(8))
        .await
        .unwrap());
    assert!(store
        .write_single_response_if_current(&transfer, 1, 0, b"newbytes")
        .await
        .unwrap());
    store_fixture::discard(&root);
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
