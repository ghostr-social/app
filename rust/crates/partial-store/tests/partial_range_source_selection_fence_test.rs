mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn selected_mirror_rejects_a_delayed_write_from_the_previous_source() {
    let root = store_fixture::temp_root("partial-source-fence");
    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let post = PostId::new("same");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(post.clone(), mirrored_meta());
    let old = binding.transfer("https://a.example/video").unwrap();
    let current = binding.transfer("https://b.example/video").unwrap();
    store.bind_representation(binding).await.unwrap();
    store.select_transfer(old.clone());
    assert!(store
        .write_range_for_transfer_if_current(&old, 0, b"kept")
        .await
        .unwrap());

    store.select_transfer(current.clone());

    assert!(!store
        .write_range_for_transfer_if_current(&old, 4, b"late")
        .await
        .unwrap());
    assert!(store
        .write_range_for_transfer_if_current(&current, 4, b"-new")
        .await
        .unwrap());
    assert_eq!(
        store.read_range("same", 0..8).await.unwrap(),
        Some(b"kept-new".to_vec())
    );
    store_fixture::discard(&root);
}

fn mirrored_meta() -> VideoMeta {
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
