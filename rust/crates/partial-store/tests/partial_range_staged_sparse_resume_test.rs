use crate::partial_range_store::ResponseOpenResult;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{ByteRange, DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn resumed_sparse_tail_remains_authorizable_while_a_whole_is_staged() {
    let root = crate::tests::store_fixture::temp_root("staged-sparse-resume");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta());
    let sparse_identity = binding
        .transfer("https://a.example/clip")
        .expect("valid test fixture");
    let whole_identity = binding
        .transfer("https://b.example/clip")
        .expect("valid test fixture");
    let generation = SourceGeneration::try_new(sparse_identity.source().as_str(), "\"a\"", 8)
        .expect("valid test fixture");
    let first = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    first
        .bind_representation(binding.clone())
        .await
        .expect("valid test fixture");
    first
        .select_transfer(sparse_identity.clone())
        .await
        .expect("valid test fixture");
    first
        .accept_generation(&sparse_identity, generation.clone())
        .await
        .expect("valid test fixture");
    first
        .write_range_for_generation_if_current(&sparse_identity, &generation, 0, b"old!")
        .await
        .expect("valid test fixture");
    drop(first);

    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    store.load_existing().await.expect("valid test fixture");
    store
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    let whole = store
        .reserve_action(&whole_identity, 1, 8)
        .await
        .expect("valid test fixture");
    assert_eq!(
        store
            .open_single_response_for_action(
                &whole_identity,
                &whole,
                crate::tests::store_fixture::exact_response(8),
            )
            .await
            .expect("valid test fixture"),
        ResponseOpenResult::Opened
    );
    let sparse = store
        .reserve_action(&sparse_identity, 2, 4)
        .await
        .expect("valid test fixture");

    assert_eq!(
        store
            .continuation_for(&sparse_identity)
            .await
            .expect("valid test fixture"),
        Some(generation.clone())
    );
    assert_eq!(
        store
            .open_sparse_response(&sparse_identity, &sparse, generation, ByteRange::new(4, 8),)
            .await
            .expect("valid test fixture"),
        ResponseOpenResult::Opened
    );
    store.release_action(&sparse).await;
    store.release_action(&whole).await;
    crate::tests::store_fixture::discard(&root);
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
