use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn sparse_reacceptance_cannot_relabel_live_whole_bytes() {
    let root = crate::tests::store_fixture::temp_root("partial-live-whole-takeover");
    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let transfer = binding
        .transfer(&meta().urls[0])
        .expect("valid test fixture");
    let generation =
        SourceGeneration::try_new(&meta().urls[0], "\"one\"", 8).expect("valid test fixture");
    store
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    store
        .select_transfer(transfer.clone())
        .await
        .expect("valid test fixture");
    store
        .accept_generation(&transfer, generation.clone())
        .await
        .expect("valid test fixture");
    store
        .begin_single_response(&transfer, 1, crate::tests::store_fixture::exact_response(8))
        .await
        .expect("valid test fixture");
    store
        .write_single_response_if_current(&transfer, 1, 0, b"whole")
        .await
        .expect("valid test fixture");

    store
        .accept_generation(&transfer, generation.clone())
        .await
        .expect("valid test fixture");

    assert_eq!(
        store
            .read_range("post", 0..5)
            .await
            .expect("valid test fixture"),
        None
    );
    assert!(store
        .write_range_for_generation_if_current(&transfer, &generation, 0, b"range")
        .await
        .expect("valid test fixture"));
    assert_eq!(
        store
            .read_range("post", 0..5)
            .await
            .expect("valid test fixture"),
        Some(b"range".to_vec())
    );
    crate::tests::store_fixture::discard(&root);
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://cdn.example/video".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
