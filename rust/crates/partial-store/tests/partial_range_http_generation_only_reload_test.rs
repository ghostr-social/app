mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

const URL: &str = "https://media.example/video.mp4";

#[tokio::test]
async fn valid_http_authority_preserves_a_live_prefix_without_legacy_state() {
    let root = store_fixture::temp_root("http-generation-only");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), metadata());
    let identity = binding.transfer(URL).unwrap();
    let first = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    first.bind_representation(binding.clone()).await.unwrap();
    first.select_transfer(identity.clone()).await.unwrap();
    first
        .apply_http_generation(&identity, store_fixture::http_generation(URL, "v1", 1))
        .await
        .unwrap();
    let action = first.reserve_action(&identity, 1, 8).await.unwrap();
    first
        .begin_single_response_for_action(&identity, &action, store_fixture::exact_response(8))
        .await
        .unwrap();
    first
        .write_single_response_for_action(&identity, &action, 0, b"part")
        .await
        .unwrap();
    drop(action);
    drop(first);

    let reopened = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.unwrap();
    reopened.bind_representation(binding).await.unwrap();
    assert_eq!(
        reopened.select_transfer(identity.clone()).await.unwrap(),
        None
    );
    reopened
        .apply_http_generation(&identity, store_fixture::http_generation(URL, "v1", 2))
        .await
        .unwrap();
    let generation = SourceGeneration::try_new(URL, "\"v1\"", 8).unwrap();
    reopened
        .accept_generation(&identity, generation)
        .await
        .unwrap();

    assert_eq!(
        reopened.read_range("post", 0..4).await.unwrap(),
        Some(b"part".to_vec())
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
