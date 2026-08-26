use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn unverified_whole_object_without_generation_is_discarded_on_restart() {
    let root = crate::tests::store_fixture::temp_root("partial-completed-binding-reload");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    store
        .bind_representation(binding.clone())
        .await
        .expect("valid test fixture");
    store
        .write_range("post", 0, b"video")
        .await
        .expect("valid test fixture");
    store
        .set_total_len("post", 5)
        .await
        .expect("valid test fixture");
    store
        .finalize("post", None)
        .await
        .expect("valid test fixture");
    drop(store);

    let reopened = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.expect("valid test fixture");
    reopened
        .bind_representation(binding)
        .await
        .expect("valid test fixture");

    assert_eq!(
        reopened
            .read_range("post", 0..5)
            .await
            .expect("valid test fixture"),
        None
    );
    assert!(!reopened
        .is_complete("post")
        .await
        .expect("valid test fixture"));
    crate::tests::store_fixture::discard(&root);
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
