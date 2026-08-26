use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

const URL: &str = "https://media.example/video.mp4";

#[tokio::test]
async fn unverified_completion_without_url_authority_is_discarded_on_restart() {
    let root = crate::tests::store_fixture::temp_root("unverified-http-generation");
    let used = Arc::new(Mutex::new(0));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), metadata());
    let identity = binding.transfer(URL).expect("valid test fixture");
    let first = crate::tests::store_fixture::plain_store(root.clone(), used);
    first
        .bind_representation(binding.clone())
        .await
        .expect("valid test fixture");
    first
        .select_transfer(identity.clone())
        .await
        .expect("valid test fixture");
    first
        .apply_http_generation(
            &identity,
            crate::tests::store_fixture::http_generation(URL, "v1", 1),
        )
        .await
        .expect("valid test fixture");
    crate::tests::store_fixture::publish_whole(&first, &identity, 1, b"oldbytes").await;
    first
        .finalize("post", None)
        .await
        .expect("valid test fixture");
    assert!(root.join("post.http-generation.json").exists());
    tokio::fs::remove_file(root.join("post.http-generation.json"))
        .await
        .expect("valid test fixture");
    drop(first);

    let reopened = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.expect("valid test fixture");
    reopened
        .bind_representation(binding)
        .await
        .expect("valid test fixture");

    assert_eq!(
        reopened
            .read_range("post", 0..8)
            .await
            .expect("valid test fixture"),
        None
    );
    assert!(!reopened
        .is_complete("post")
        .await
        .expect("valid test fixture"));
    crate::tests::store_fixture::discard(&root);
}

fn metadata() -> VideoMeta {
    VideoMeta {
        urls: vec![URL.into()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
