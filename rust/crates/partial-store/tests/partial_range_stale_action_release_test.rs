use crate::partial_range_store::ResponseOpenResult;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn stale_same_id_action_cannot_abort_the_new_response_authority() {
    let root = crate::tests::store_fixture::temp_root("stale-action-release");
    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let identity = binding
        .transfer("https://cdn.example/video")
        .expect("valid test fixture");
    store
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    store
        .select_transfer(identity.clone())
        .await
        .expect("valid test fixture");
    store
        .write_range("post", 0, b"old!")
        .await
        .expect("valid test fixture");
    let stale = store
        .reserve_action(&identity, 1, 8)
        .await
        .expect("valid test fixture");
    assert_eq!(
        store
            .open_single_response_for_action(
                &identity,
                &stale,
                crate::tests::store_fixture::exact_response(8),
            )
            .await
            .expect("valid test fixture"),
        ResponseOpenResult::Opened
    );
    store.release_action(&stale).await;
    let current = store
        .reserve_action(&identity, 1, 8)
        .await
        .expect("valid test fixture");
    assert_eq!(
        store
            .open_single_response_for_action(
                &identity,
                &current,
                crate::tests::store_fixture::exact_response(8),
            )
            .await
            .expect("valid test fixture"),
        ResponseOpenResult::Opened
    );

    store.release_action(&stale).await;

    assert!(store
        .write_single_response_for_action(&identity, &current, 0, b"new!")
        .await
        .expect("valid test fixture"));
    store.release_action(&current).await;
    crate::tests::store_fixture::discard(&root);
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://cdn.example/video".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
