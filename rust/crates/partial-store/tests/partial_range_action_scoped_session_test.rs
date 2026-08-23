mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_completion::Completion;
use std::sync::Arc;
use tokio::sync::Mutex;

const URL: &str = "https://media.example/video.mp4";

#[tokio::test]
async fn action_scoped_whole_is_session_readable_but_not_restart_reusable() {
    let root = store_fixture::temp_root("action-scoped-session");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), metadata());
    let identity = binding.transfer(URL).unwrap();
    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    store.bind_representation(binding.clone()).await.unwrap();
    store.select_transfer(identity.clone()).await.unwrap();
    let action = store.reserve_action(&identity, 1, 8).await.unwrap();
    store
        .open_action_scoped_single_response(&identity, &action, store_fixture::exact_response(8))
        .await
        .unwrap();
    store
        .write_single_response_for_action(&identity, &action, 0, b"newbytes")
        .await
        .unwrap();
    assert!(store
        .finish_single_response_for_action(&identity, &action, Some(8), true)
        .await
        .unwrap());

    assert_eq!(
        store.finalize("post", None).await.unwrap(),
        Completion::Unverified
    );
    assert_eq!(
        store.read_range("post", 0..8).await.unwrap(),
        Some(b"newbytes".to_vec())
    );
    drop(store);
    let reopened = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.unwrap();
    reopened.bind_representation(binding).await.unwrap();
    assert_eq!(reopened.read_range("post", 0..8).await.unwrap(), None);
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
