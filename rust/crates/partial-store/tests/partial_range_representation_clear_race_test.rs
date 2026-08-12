#[path = "store_fixture/paused.rs"]
mod paused_fixture;
mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn clearing_during_a_write_discards_bytes_without_restoring_a_binding() {
    let mut fixture = paused_fixture::paused_store("partial-representation-clear-race");
    let store = fixture.store.clone();
    let mut catalog = Catalog::new();
    let post = PostId::new("clip");
    let binding = catalog.upsert(post.clone(), meta());
    let transfer = catalog
        .transfer_identity(&post, "https://video.example/clip.mp4")
        .unwrap();
    store.bind_representation(binding).await.unwrap();
    store.select_transfer(transfer.clone());

    let writer = tokio::spawn({
        let store = store.clone();
        async move {
            store
                .write_range_for_transfer_if_current(&transfer, 0, b"data")
                .await
        }
    });
    fixture.wait_until_admission().await;
    let clearing = tokio::spawn({
        let store = store.clone();
        async move { store.clear().await }
    });
    wait_until_binding_is_cleared(&store).await;
    fixture.resume();

    assert!(!writer.await.unwrap().unwrap());
    clearing.await.unwrap().unwrap();
    assert!(store.representation_binding("clip").await.is_none());
    assert!(store.present_ranges("clip").await.unwrap().is_empty());
    store_fixture::discard(&fixture.root);
}

async fn wait_until_binding_is_cleared(
    store: &Arc<ghostr_partial_store::partial_range_store::PartialRangeStore>,
) {
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
