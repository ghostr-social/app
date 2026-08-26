use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

const URL: &str = "https://media.example/video.mp4";

#[tokio::test]
async fn newer_http_generation_removes_an_old_staged_response() {
    let root = crate::tests::store_fixture::temp_root("staged-http-generation");
    let used = Arc::new(Mutex::new(0));
    let store =
        crate::tests::store_fixture::plain_store(root.clone(), std::sync::Arc::clone(&used));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), metadata());
    let identity = binding.transfer(URL).expect("valid test fixture");
    store
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    store
        .select_transfer(identity.clone())
        .await
        .expect("valid test fixture");
    let v1 = crate::tests::store_fixture::http_generation(URL, "v1", 1);
    store
        .apply_http_generation(&identity, v1.clone())
        .await
        .expect("valid test fixture");
    crate::tests::store_fixture::publish_whole(&store, &identity, 1, b"oldbytes").await;
    let stale = store
        .reserve_action(&identity, 2, 8)
        .await
        .expect("valid test fixture");
    assert!(store
        .begin_single_response_for_action(
            &identity,
            &stale,
            crate::tests::store_fixture::exact_response(8),
        )
        .await
        .expect("valid test fixture"));
    assert!(store
        .write_single_response_for_action(&identity, &stale, 0, b"new!")
        .await
        .expect("valid test fixture"));
    assert_eq!(
        store
            .read_range("post", 0..8)
            .await
            .expect("valid test fixture"),
        Some(b"oldbytes".to_vec())
    );

    let v2 = crate::tests::store_fixture::http_generation(URL, "v2", 2);
    store
        .apply_http_generation(&identity, v2)
        .await
        .expect("valid test fixture");

    assert!(!stale.is_active());
    assert_eq!(
        store
            .read_range("post", 0..8)
            .await
            .expect("valid test fixture"),
        None
    );
    assert!(!root.join("post.response.part").exists());
    assert_eq!(*used.lock().await, 0);
    crate::tests::store_fixture::discard(&root);
}

fn metadata() -> VideoMeta {
    VideoMeta {
        urls: vec![URL.into()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
