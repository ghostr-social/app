use crate::partial_range_store::ResponseOpenResult;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

const URL: &str = "https://media.example/video.mp4";

#[tokio::test]
async fn action_scoped_whole_never_publishes_under_prior_authority() {
    let root = crate::tests::store_fixture::temp_root("action-scoped-replacement");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), metadata());
    let identity = binding.transfer(URL).expect("valid test fixture");
    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    store
        .bind_representation(binding.clone())
        .await
        .expect("valid test fixture");
    store
        .select_transfer(identity.clone())
        .await
        .expect("valid test fixture");
    store
        .apply_http_generation(
            &identity,
            crate::tests::store_fixture::http_generation(URL, "v1", 1),
        )
        .await
        .expect("valid test fixture");
    crate::tests::store_fixture::publish_whole(&store, &identity, 1, b"oldbytes").await;
    store
        .finalize("post", None)
        .await
        .expect("valid test fixture");

    let action = store
        .reserve_action(&identity, 2, 8)
        .await
        .expect("valid test fixture");
    assert!(matches!(
        store
            .open_action_scoped_single_response(
                &identity,
                &action,
                crate::tests::store_fixture::exact_response(8)
            )
            .await
            .expect("valid test fixture"),
        ResponseOpenResult::Opened
    ));
    assert!(store
        .write_single_response_for_action(&identity, &action, 0, b"newbytes")
        .await
        .expect("valid test fixture"));
    assert_eq!(
        store
            .read_range("post", 0..8)
            .await
            .expect("valid test fixture"),
        Some(b"oldbytes".to_vec())
    );
    assert!(root.join("post.http-generation.json").exists());

    assert!(store
        .finish_single_response_for_action(&identity, &action, Some(8), true)
        .await
        .expect("valid test fixture"));
    assert_eq!(
        store
            .read_range("post", 0..8)
            .await
            .expect("valid test fixture"),
        Some(b"newbytes".to_vec())
    );
    assert!(root.join("post.http-generation.json").exists());
    assert!(root.join("post.video").exists());
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
            .read_range("post", 0..8)
            .await
            .expect("valid test fixture"),
        Some(b"oldbytes".to_vec())
    );
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
