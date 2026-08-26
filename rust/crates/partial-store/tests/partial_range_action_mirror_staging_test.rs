use crate::partial_range_store::ResponseOpenResult;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn failed_mirror_response_never_replaces_the_canonical_generation() {
    let root = crate::tests::store_fixture::temp_root("action-mirror-stage");
    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let primary = binding
        .transfer("https://a.example/video")
        .expect("valid test fixture");
    let mirror = binding
        .transfer("https://b.example/video")
        .expect("valid test fixture");
    let generation = SourceGeneration::try_new(primary.source().as_str(), "\"a\"", 8)
        .expect("valid test fixture");
    store
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    store
        .select_transfer(primary.clone())
        .await
        .expect("valid test fixture");
    store
        .accept_generation(&primary, generation.clone())
        .await
        .expect("valid test fixture");
    store
        .write_range_for_generation_if_current(&primary, &generation, 0, b"old!")
        .await
        .expect("valid test fixture");
    let action = store
        .reserve_action(&mirror, 7, 8)
        .await
        .expect("valid test fixture");

    assert_eq!(
        store
            .open_single_response_for_action(
                &mirror,
                &action,
                crate::tests::store_fixture::exact_response(8),
            )
            .await
            .expect("valid test fixture"),
        ResponseOpenResult::Opened
    );
    assert!(store
        .write_single_response_for_action(&mirror, &action, 0, b"new!")
        .await
        .expect("valid test fixture"));
    assert_eq!(
        store
            .read_range("post", 0..4)
            .await
            .expect("valid test fixture"),
        Some(b"old!".to_vec())
    );
    assert!(!store
        .finish_single_response_for_action(&mirror, &action, Some(8), false)
        .await
        .expect("valid test fixture"));
    assert!(store.transfer_is_current(&primary).await);
    store.release_action(&action).await;
    crate::tests::store_fixture::discard(&root);
}

fn meta() -> VideoMeta {
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
