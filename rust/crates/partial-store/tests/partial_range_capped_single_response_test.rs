mod store_fixture;

use ghostr_engine::adaptive::WholeBodyContract;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn capped_response_keeps_seed_until_eof_then_commits_discovered_total() {
    let root = store_fixture::temp_root("single-response-capped");
    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let transfer = binding.transfer("https://cdn.example/video").unwrap();
    store.bind_representation(binding).await.unwrap();
    store.select_transfer(transfer.clone()).await.unwrap();
    store.write_range("post", 0, b"old!").await.unwrap();

    assert!(store
        .begin_single_response(
            &transfer,
            7,
            WholeBodyContract::Capped { maximum_bytes: 16 },
        )
        .await
        .unwrap());
    store
        .write_single_response_if_current(&transfer, 7, 0, b"new body")
        .await
        .unwrap();
    assert_eq!(
        store.read_range("post", 0..4).await.unwrap(),
        Some(b"old!".to_vec())
    );

    assert!(store
        .finish_single_response(&transfer, 7, Some(8), true)
        .await
        .unwrap());
    assert_eq!(store.total_len("post").await.unwrap(), Some(8));
    assert_eq!(
        store.read_range("post", 0..8).await.unwrap(),
        Some(b"new body".to_vec())
    );
    store_fixture::discard(&root);
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://cdn.example/video".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: None,
        duration_ms: Some(1_000),
    }
}
