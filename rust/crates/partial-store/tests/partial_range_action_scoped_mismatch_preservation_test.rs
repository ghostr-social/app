use crate::partial_range_completion::IntegrityMismatch;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::sync::Arc;
use tokio::sync::Mutex;

const URL: &str = "https://media.example/video.mp4";

#[tokio::test]
async fn mismatched_action_scoped_whole_preserves_prior_finalized_video() {
    let root = crate::tests::store_fixture::temp_root("action-scoped-mismatch");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), metadata());
    let identity = binding.transfer(URL).expect("valid test fixture");
    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    store
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    store
        .select_transfer(identity.clone())
        .await
        .expect("valid test fixture");
    crate::tests::store_fixture::publish_whole(&store, &identity, 1, b"oldbytes").await;
    store
        .finalize("post", None)
        .await
        .expect("valid test fixture");
    let action = store
        .reserve_action(&identity, 2, 8)
        .await
        .expect("valid test fixture");
    store
        .open_action_scoped_single_response(
            &identity,
            &action,
            crate::tests::store_fixture::exact_response(8),
        )
        .await
        .expect("valid test fixture");
    store
        .write_single_response_for_action(&identity, &action, 0, b"newbytes")
        .await
        .expect("valid test fixture");
    store
        .finish_single_response_for_action(&identity, &action, Some(8), true)
        .await
        .expect("valid test fixture");

    let error = store
        .finalize("post", Some(&"a".repeat(64)))
        .await
        .expect_err("scenario must fail");
    assert!(error.downcast_ref::<IntegrityMismatch>().is_some());
    assert_eq!(
        store
            .read_range("post", 0..8)
            .await
            .expect("valid test fixture"),
        Some(b"oldbytes".to_vec())
    );
    assert!(root.join("post.video").exists());
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
