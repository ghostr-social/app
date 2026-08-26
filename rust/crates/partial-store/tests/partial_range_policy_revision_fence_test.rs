use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn stale_policy_plan_never_evicts_replacement_bytes() {
    let root = crate::tests::store_fixture::temp_root("partial-policy-revision-fence");
    let store = crate::tests::store_fixture::plain_store(root.clone(), Default::default());
    let mut catalog = Catalog::new();
    store
        .bind_representation(catalog.upsert(PostId::new("post"), meta("old")))
        .await
        .expect("valid test fixture");
    store
        .write_range("post", 0, b"oldbytes")
        .await
        .expect("valid test fixture");
    let revision = store
        .media_snapshot("post")
        .await
        .expect("valid test fixture")
        .revision();

    store
        .bind_representation(catalog.upsert(PostId::new("post"), meta("new")))
        .await
        .expect("valid test fixture");
    store
        .write_range("post", 0, b"newbytes")
        .await
        .expect("valid test fixture");
    let requested = 0..8;
    let outcome = store
        .evict_ranges_if_current("post", core::slice::from_ref(&requested), revision)
        .await
        .expect("valid test fixture");

    assert_eq!(outcome.freed_bytes(), 0);
    assert!(outcome.ranges().is_empty());
    assert_eq!(
        store
            .read_range("post", 0..8)
            .await
            .expect("valid test fixture"),
        Some(b"newbytes".to_vec())
    );
    crate::tests::store_fixture::discard(&root);
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
