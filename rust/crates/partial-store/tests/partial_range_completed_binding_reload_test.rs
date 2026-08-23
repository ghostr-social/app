mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn unverified_whole_object_without_generation_is_discarded_on_restart() {
    let root = store_fixture::temp_root("partial-completed-binding-reload");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    store.bind_representation(binding.clone()).await.unwrap();
    store.write_range("post", 0, b"video").await.unwrap();
    store.set_total_len("post", 5).await.unwrap();
    store.finalize("post", None).await.unwrap();
    drop(store);

    let reopened = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.unwrap();
    reopened.bind_representation(binding).await.unwrap();

    assert_eq!(reopened.read_range("post", 0..5).await.unwrap(), None);
    assert!(!reopened.is_complete("post").await.unwrap());
    store_fixture::discard(&root);
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://cdn.example/video".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(5),
        duration_ms: Some(1_000),
    }
}
