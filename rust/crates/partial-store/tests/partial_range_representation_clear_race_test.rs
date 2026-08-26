use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn clearing_during_a_write_discards_bytes_without_restoring_a_binding() {
    let mut fixture =
        crate::tests::paused_fixture::paused_store("partial-representation-clear-race");
    let store = std::sync::Arc::clone(&fixture.store);
    let mut catalog = Catalog::new();
    let post = PostId::new("clip");
    let binding = catalog.upsert(post.clone(), meta());
    let transfer = catalog
        .transfer_identity(&post, "https://video.example/clip.mp4")
        .expect("valid test fixture");
    store
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    store
        .select_transfer(transfer.clone())
        .await
        .expect("valid test fixture");

    let writer = tokio::spawn({
        let store = std::sync::Arc::clone(&store);
        async move {
            store
                .write_range_for_transfer_if_current(&transfer, 0, b"data")
                .await
        }
    });
    fixture.wait_until_admission().await;
    let clearing = tokio::spawn({
        let store = std::sync::Arc::clone(&store);
        async move { store.clear().await }
    });
    wait_until_binding_is_cleared(&store).await;
    fixture.resume();

    assert!(!writer
        .await
        .expect("valid test fixture")
        .expect("valid test fixture"));
    clearing
        .await
        .expect("valid test fixture")
        .expect("valid test fixture");
    assert!(store.representation_binding("clip").await.is_none());
    assert!(store
        .present_ranges("clip")
        .await
        .expect("valid test fixture")
        .is_empty());
    crate::tests::store_fixture::discard(&fixture.root);
}

async fn wait_until_binding_is_cleared(store: &Arc<crate::partial_range_store::PartialRangeStore>) {
    while store.representation_binding("clip").await.is_some() {
        tokio::task::yield_now().await;
    }
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://video.example/clip.mp4".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
