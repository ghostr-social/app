use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

const URL: &str = "https://media.example/video.mp4";

#[tokio::test]
async fn valid_http_authority_preserves_a_live_prefix_without_legacy_state() {
    let root = crate::tests::store_fixture::temp_root("http-generation-only");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), metadata());
    let identity = binding.transfer(URL).expect("valid test fixture");
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
        .apply_http_generation(
            &identity,
            crate::tests::store_fixture::http_generation(URL, "v1", 1),
        )
        .await
        .expect("valid test fixture");
    let action = first
        .reserve_action(&identity, 1, 8)
        .await
        .expect("valid test fixture");
    first
        .begin_single_response_for_action(
            &identity,
            &action,
            crate::tests::store_fixture::exact_response(8),
        )
        .await
        .expect("valid test fixture");
    first
        .write_single_response_for_action(&identity, &action, 0, b"part")
        .await
        .expect("valid test fixture");
    drop(action);
    drop(first);

    let reopened = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.expect("valid test fixture");
    reopened
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    assert_eq!(
        reopened
            .select_transfer(identity.clone())
            .await
            .expect("valid test fixture"),
        None
    );
    reopened
        .apply_http_generation(
            &identity,
            crate::tests::store_fixture::http_generation(URL, "v1", 2),
        )
        .await
        .expect("valid test fixture");
    let generation = SourceGeneration::try_new(URL, "\"v1\"", 8).expect("valid test fixture");
    reopened
        .accept_generation(&identity, generation)
        .await
        .expect("valid test fixture");

    assert_eq!(
        reopened
            .read_range("post", 0..4)
            .await
            .expect("valid test fixture"),
        Some(b"part".to_vec())
    );
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
