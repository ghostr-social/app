mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{ByteRange, DeliveryKind, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::ResponseOpenResult;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn resumed_sparse_tail_remains_authorizable_while_a_whole_is_staged() {
    let root = store_fixture::temp_root("staged-sparse-resume");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta());
    let sparse_identity = binding.transfer("https://a.example/clip").unwrap();
    let whole_identity = binding.transfer("https://b.example/clip").unwrap();
    let generation =
        SourceGeneration::try_new(sparse_identity.source().as_str(), "\"a\"", 8).unwrap();
    let first = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    first.bind_representation(binding.clone()).await.unwrap();
    first
        .select_transfer(sparse_identity.clone())
        .await
        .unwrap();
    first
        .accept_generation(&sparse_identity, generation.clone())
        .await
        .unwrap();
    first
        .write_range_for_generation_if_current(&sparse_identity, &generation, 0, b"old!")
        .await
        .unwrap();
    drop(first);

    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    store.load_existing().await.unwrap();
    store.bind_representation(binding).await.unwrap();
    let whole = store.reserve_action(&whole_identity, 1, 8).await.unwrap();
    assert_eq!(
        store
            .open_single_response_for_action(
                &whole_identity,
                &whole,
                store_fixture::exact_response(8),
            )
            .await
            .unwrap(),
        ResponseOpenResult::Opened
    );
    let sparse = store.reserve_action(&sparse_identity, 2, 4).await.unwrap();

    assert_eq!(
        store.continuation_for(&sparse_identity).await.unwrap(),
        Some(generation.clone())
    );
    assert_eq!(
        store
            .open_sparse_response(&sparse_identity, &sparse, generation, ByteRange::new(4, 8),)
            .await
            .unwrap(),
        ResponseOpenResult::Opened
    );
    store.release_action(&sparse).await;
    store.release_action(&whole).await;
    store_fixture::discard(&root);
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec![
            "https://a.example/clip".to_owned(),
            "https://b.example/clip".to_owned(),
        ],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
