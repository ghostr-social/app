mod store_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn stale_policy_plan_never_evicts_replacement_bytes() {
    let root = store_fixture::temp_root("partial-policy-revision-fence");
    let store = store_fixture::plain_store(root.clone(), Default::default());
    let mut catalog = Catalog::new();
    store
        .bind_representation(catalog.upsert(PostId::new("post"), meta("old")))
        .await
        .unwrap();
    store.write_range("post", 0, b"oldbytes").await.unwrap();
    let revision = store.media_snapshot("post").await.unwrap().revision();

    store
        .bind_representation(catalog.upsert(PostId::new("post"), meta("new")))
        .await
        .unwrap();
    store.write_range("post", 0, b"newbytes").await.unwrap();
    let requested = 0..8;
    let outcome = store
        .evict_ranges_if_current("post", std::slice::from_ref(&requested), revision)
        .await
        .unwrap();

    assert_eq!(outcome.freed_bytes(), 0);
    assert!(outcome.ranges().is_empty());
    assert_eq!(
        store.read_range("post", 0..8).await.unwrap(),
        Some(b"newbytes".to_vec())
    );
    store_fixture::discard(&root);
}

fn meta(name: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![format!("https://{name}.example/video")],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
