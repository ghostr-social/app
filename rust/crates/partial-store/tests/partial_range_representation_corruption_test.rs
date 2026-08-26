use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn corrupt_representation_identity_fails_closed() {
    let root = crate::tests::store_fixture::temp_root("partial-representation-corrupt");
    tokio::fs::create_dir_all(&root)
        .await
        .expect("valid test fixture");
    tokio::fs::write(root.join("same.representation"), "not-a-fingerprint")
        .await
        .expect("valid test fixture");
    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let (binding, identity) = binding();

    let error = store
        .bind_representation(binding)
        .await
        .expect_err("corrupt sidecar must be rejected");
    let _ = store.select_transfer(identity.clone()).await;

    assert!(error.to_string().contains("identity is invalid"));
    assert!(!store.transfer_is_current(&identity).await);
    crate::tests::store_fixture::discard(&root);
}

fn binding() -> (
    ghostr_engine::representation::RepresentationBinding,
    ghostr_engine::representation::TransferIdentity,
) {
    let post = PostId::new("same");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(post, meta());
    let identity = binding
        .transfer("https://a.example/video")
        .expect("valid test fixture");
    (binding, identity)
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://a.example/video".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
