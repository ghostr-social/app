mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_completion::Completion;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio::sync::Mutex;

const URL: &str = "https://media.example/video.mp4";

#[tokio::test]
async fn matching_action_scoped_whole_promotes_and_survives_restart() {
    let root = store_fixture::temp_root("action-scoped-verified");
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
    store
        .finish_single_response_for_action(&identity, &action, Some(8), true)
        .await
        .unwrap();
    let digest = format!("{:x}", Sha256::digest(b"newbytes"));
    assert_eq!(
        store.finalize("post", Some(&digest)).await.unwrap(),
        Completion::Verified
    );
    drop(store);

    let reopened = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.unwrap();
    reopened.bind_representation(binding).await.unwrap();
    assert_eq!(
        reopened.read_range("post", 0..8).await.unwrap(),
        Some(b"newbytes".to_vec())
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
