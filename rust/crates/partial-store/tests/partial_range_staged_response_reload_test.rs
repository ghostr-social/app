mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::ResponseOpenResult;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn restart_discards_uncommitted_mirror_stage_and_recovers_canonical_bytes() {
    let root = store_fixture::temp_root("staged-response-reload");
    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let primary = binding.transfer("https://a.example/video").unwrap();
    let mirror = binding.transfer("https://b.example/video").unwrap();
    let generation = SourceGeneration::try_new(primary.source().as_str(), "\"a\"", 8).unwrap();
    store.bind_representation(binding.clone()).await.unwrap();
    store.select_transfer(primary.clone()).await.unwrap();
    store
        .accept_generation(&primary, generation.clone())
        .await
        .unwrap();
    store
        .write_range_for_generation_if_current(&primary, &generation, 0, b"old!")
        .await
        .unwrap();
    let action = store.reserve_action(&mirror, 1, 8).await.unwrap();
    assert_eq!(
        store
            .open_single_response_for_action(&mirror, &action, store_fixture::exact_response(8))
            .await
            .unwrap(),
        ResponseOpenResult::Opened
    );
    store
        .write_single_response_for_action(&mirror, &action, 0, b"new!")
        .await
        .unwrap();
    drop(store);
    drop(action);

    let reopened = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.unwrap();
    reopened.bind_representation(binding).await.unwrap();
    let restored = reopened.select_transfer(primary).await.unwrap();

    assert_eq!(
        reopened.read_range("post", 0..4).await.unwrap(),
        Some(b"old!".to_vec())
    );
    assert_eq!(restored, Some(generation));
    store_fixture::discard(&root);
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec![
            "https://a.example/video".to_owned(),
            "https://b.example/video".to_owned(),
        ],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
