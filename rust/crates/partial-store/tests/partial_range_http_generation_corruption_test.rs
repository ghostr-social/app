mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

const URL: &str = "https://media.example/video.mp4";

#[tokio::test]
async fn malformed_http_authority_never_falls_back_to_legacy_bytes() {
    let root = store_fixture::temp_root("http-generation-corruption");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), metadata());
    let identity = binding.transfer(URL).unwrap();
    let generation = SourceGeneration::try_new(URL, "\"v1\"", 8).unwrap();
    let first = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    first.bind_representation(binding.clone()).await.unwrap();
    first.select_transfer(identity.clone()).await.unwrap();
    first
        .accept_generation(&identity, generation.clone())
        .await
        .unwrap();
    first
        .write_range_for_generation_if_current(&identity, &generation, 0, b"part")
        .await
        .unwrap();
    tokio::fs::write(root.join("post.http-generation.json"), b"{")
        .await
        .unwrap();
    drop(first);

    let used = Arc::new(Mutex::new(0));
    let reopened = store_fixture::plain_store(root.clone(), used.clone());
    reopened.load_existing().await.unwrap();
    reopened.bind_representation(binding).await.unwrap();

    assert_eq!(reopened.read_range("post", 0..4).await.unwrap(), None);
    assert_eq!(*used.lock().await, 0);
    assert!(!root.join("post.http-generation.json").exists());
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
