mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn stale_writer_cannot_mix_bytes_after_representation_replacement() {
    let root = store_fixture::temp_root("partial-representation-fence");
    let used = Arc::new(Mutex::new(0));
    let store = store_fixture::plain_store(root.clone(), used.clone());
    let post = PostId::new("same");
    let mut catalog = Catalog::new();

    let first = catalog.upsert(post.clone(), meta("https://a.example/video", 8));
    let stale = catalog
        .transfer_identity(&post, "https://a.example/video")
        .unwrap();
    store.bind_representation(first).await.unwrap();
    store.select_transfer(stale.clone());
    assert!(store
        .write_range_for_transfer_if_current(&stale, 0, b"old")
        .await
        .unwrap());

    let second = catalog.upsert(post.clone(), meta("https://b.example/video", 4));
    let current = catalog
        .transfer_identity(&post, "https://b.example/video")
        .unwrap();
    store.bind_representation(second).await.unwrap();
    store.select_transfer(current.clone());
    assert!(!store
        .write_range_for_transfer_if_current(&stale, 0, b"late")
        .await
        .unwrap());
    assert!(store
        .write_range_for_transfer_if_current(&current, 0, b"new!")
        .await
        .unwrap());
    assert_eq!(
        store.read_range("same", 0..4).await.unwrap(),
        Some(b"new!".to_vec())
    );

    drop(store);
    let reopened = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.unwrap();
    reopened
        .bind_representation(catalog.binding(&post).unwrap())
        .await
        .unwrap();
    assert_eq!(
        reopened.read_range("same", 0..4).await.unwrap(),
        Some(b"new!".to_vec())
    );
    store_fixture::discard(&root);
}

fn meta(url: &str, size: u64) -> VideoMeta {
    VideoMeta {
        urls: vec![url.to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(size),
        duration_ms: Some(1_000),
    }
}
