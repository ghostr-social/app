use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn selected_mirror_rejects_a_delayed_write_from_the_previous_source() {
    let root = crate::tests::store_fixture::temp_root("partial-source-fence");
    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let post = PostId::new("same");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(post.clone(), mirrored_meta());
    let old = binding
        .transfer("https://a.example/video")
        .expect("valid test fixture");
    let current = binding
        .transfer("https://b.example/video")
        .expect("valid test fixture");
    store
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    store
        .select_transfer(old.clone())
        .await
        .expect("valid test fixture");
    assert!(store
        .write_range_for_transfer_if_current(&old, 0, b"kept")
        .await
        .expect("valid test fixture"));

    store
        .select_transfer(current.clone())
        .await
        .expect("valid test fixture");

    assert!(!store
        .write_range_for_transfer_if_current(&old, 4, b"late")
        .await
        .expect("valid test fixture"));
    assert!(store
        .write_range_for_transfer_if_current(&current, 4, b"new!")
        .await
        .expect("valid test fixture"));
    assert_eq!(
        store
            .read_range("same", 0..4)
            .await
            .expect("valid test fixture"),
        None
    );
    assert_eq!(
        store
            .read_range("same", 4..8)
            .await
            .expect("valid test fixture"),
        Some(b"new!".to_vec())
    );
    crate::tests::store_fixture::discard(&root);
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
