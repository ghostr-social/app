use crate::partial_range_completion::Completion;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use sha2::{Digest as _, Sha256};
use std::sync::Arc;
use tokio::sync::Mutex;

const URL: &str = "https://media.example/video.mp4";

#[tokio::test]
async fn matching_action_scoped_whole_promotes_and_survives_restart() {
    let root = crate::tests::store_fixture::temp_root("action-scoped-verified");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), metadata());
    let identity = binding.transfer(URL).expect("valid test fixture");
    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    store
        .bind_representation(binding.clone())
        .await
        .expect("valid test fixture");
    store
        .select_transfer(identity.clone())
        .await
        .expect("valid test fixture");
    let action = store
        .reserve_action(&identity, 1, 8)
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
    let digest = format!("{:x}", Sha256::digest(b"newbytes"));
    assert_eq!(
        store
            .finalize("post", Some(&digest))
            .await
            .expect("valid test fixture"),
        Completion::Verified
    );
    drop(store);

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
        Some(b"newbytes".to_vec())
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
