mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio::sync::Mutex;

const URL: &str = "https://media.example/video.mp4";

#[tokio::test]
async fn verified_completion_retires_url_authority_and_survives_changes() {
    let root = store_fixture::temp_root("verified-http-generation");
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
    let digest = format!("{:x}", Sha256::digest(b"oldbytes"));

    store.finalize("post", Some(&digest)).await.unwrap();

    assert!(!root.join("post.http-generation.json").exists());
    drop(store);
    let reopened = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.unwrap();
    reopened.bind_representation(binding).await.unwrap();
    let applied = reopened
        .apply_http_generation(&identity, store_fixture::http_generation(URL, "v2", 2))
        .await
        .unwrap();
    assert!(
        !applied,
        "verified bytes cannot adopt mutable URL authority"
    );
    assert_eq!(
        reopened.read_range("post", 0..8).await.unwrap(),
        Some(b"oldbytes".to_vec())
    );
    assert!(reopened.is_complete("post").await.unwrap());
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
