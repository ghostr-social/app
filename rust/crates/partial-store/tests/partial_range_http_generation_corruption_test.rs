use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

const URL: &str = "https://media.example/video.mp4";

#[tokio::test]
async fn malformed_http_authority_never_falls_back_to_legacy_bytes() {
    let root = crate::tests::store_fixture::temp_root("http-generation-corruption");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), metadata());
    let identity = binding.transfer(URL).expect("valid test fixture");
    let generation = SourceGeneration::try_new(URL, "\"v1\"", 8).expect("valid test fixture");
    let first = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    first
        .bind_representation(binding.clone())
        .await
        .expect("valid test fixture");
    first
        .select_transfer(identity.clone())
        .await
        .expect("valid test fixture");
    first
        .accept_generation(&identity, generation.clone())
        .await
        .expect("valid test fixture");
    first
        .write_range_for_generation_if_current(&identity, &generation, 0, b"part")
        .await
        .expect("valid test fixture");
    tokio::fs::write(root.join("post.http-generation.json"), b"{")
        .await
        .expect("valid test fixture");
    drop(first);

    let used = Arc::new(Mutex::new(0));
    let reopened =
        crate::tests::store_fixture::plain_store(root.clone(), std::sync::Arc::clone(&used));
    reopened.load_existing().await.expect("valid test fixture");
    reopened
        .bind_representation(binding)
        .await
        .expect("valid test fixture");

    assert_eq!(
        reopened
            .read_range("post", 0..4)
            .await
            .expect("valid test fixture"),
        None
    );
    assert_eq!(*used.lock().await, 0);
    assert!(!root.join("post.http-generation.json").exists());
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
