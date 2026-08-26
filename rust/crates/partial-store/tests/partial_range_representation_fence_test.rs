use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn stale_writer_cannot_mix_bytes_after_representation_replacement() {
    let root = crate::tests::store_fixture::temp_root("partial-representation-fence");
    let used = Arc::new(Mutex::new(0));
    let store =
        crate::tests::store_fixture::plain_store(root.clone(), std::sync::Arc::clone(&used));
    let post = PostId::new("same");
    let mut catalog = Catalog::new();

    let first = catalog.upsert(post.clone(), meta("https://a.example/video", 8));
    let stale = catalog
        .transfer_identity(&post, "https://a.example/video")
        .expect("valid test fixture");
    store
        .bind_representation(first)
        .await
        .expect("valid test fixture");
    store
        .select_transfer(stale.clone())
        .await
        .expect("valid test fixture");
    assert!(store
        .write_range_for_transfer_if_current(&stale, 0, b"old")
        .await
        .expect("valid test fixture"));

    let second = catalog.upsert(post.clone(), meta("https://b.example/video", 4));
    let current = catalog
        .transfer_identity(&post, "https://b.example/video")
        .expect("valid test fixture");
    store
        .bind_representation(second)
        .await
        .expect("valid test fixture");
    store
        .select_transfer(current.clone())
        .await
        .expect("valid test fixture");
    let generation = SourceGeneration::try_new("https://b.example/video", "\"current\"", 4)
        .expect("valid test fixture");
    store
        .accept_generation(&current, generation.clone())
        .await
        .expect("valid test fixture");
    assert!(!store
        .write_range_for_transfer_if_current(&stale, 0, b"late")
        .await
        .expect("valid test fixture"));
    assert!(store
        .write_range_for_generation_if_current(&current, &generation, 0, b"new!")
        .await
        .expect("valid test fixture"));
    assert_eq!(
        store
            .read_range("same", 0..4)
            .await
            .expect("valid test fixture"),
        Some(b"new!".to_vec())
    );

    drop(store);
    let reopened = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.expect("valid test fixture");
    reopened
        .bind_representation(catalog.binding(&post).expect("valid test fixture"))
        .await
        .expect("valid test fixture");
    assert_eq!(
        reopened
            .read_range("same", 0..4)
            .await
            .expect("valid test fixture"),
        Some(b"new!".to_vec())
    );
    crate::tests::store_fixture::discard(&root);
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
