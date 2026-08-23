mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::ResponseOpenResult;
use std::sync::Arc;
use tokio::sync::Mutex;

const URL: &str = "https://media.example/video.mp4";

#[tokio::test]
async fn action_scoped_whole_never_publishes_under_prior_authority() {
    let root = store_fixture::temp_root("action-scoped-replacement");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), metadata());
    let identity = binding.transfer(URL).unwrap();
    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    store.bind_representation(binding.clone()).await.unwrap();
    store.select_transfer(identity.clone()).await.unwrap();
    store
        .apply_http_generation(&identity, store_fixture::http_generation(URL, "v1", 1))
        .await
        .unwrap();
    store_fixture::publish_whole(&store, &identity, 1, b"oldbytes").await;
    store.finalize("post", None).await.unwrap();

    let action = store.reserve_action(&identity, 2, 8).await.unwrap();
    assert!(matches!(
        store
            .open_action_scoped_single_response(
                &identity,
                &action,
                store_fixture::exact_response(8)
            )
            .await
            .unwrap(),
        ResponseOpenResult::Opened
    ));
    assert!(store
        .write_single_response_for_action(&identity, &action, 0, b"newbytes")
        .await
        .unwrap());
    assert_eq!(
        store.read_range("post", 0..8).await.unwrap(),
        Some(b"oldbytes".to_vec())
    );
    assert!(root.join("post.http-generation.json").exists());

    assert!(store
        .finish_single_response_for_action(&identity, &action, Some(8), true)
        .await
        .unwrap());
    assert_eq!(
        store.read_range("post", 0..8).await.unwrap(),
        Some(b"newbytes".to_vec())
    );
    assert!(root.join("post.http-generation.json").exists());
    assert!(root.join("post.video").exists());
    store.finalize("post", None).await.unwrap();
    drop(store);

    let reopened = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.unwrap();
    reopened.bind_representation(binding).await.unwrap();
    assert_eq!(
        reopened.read_range("post", 0..8).await.unwrap(),
        Some(b"oldbytes".to_vec())
    );
    store_fixture::discard(&root);
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
