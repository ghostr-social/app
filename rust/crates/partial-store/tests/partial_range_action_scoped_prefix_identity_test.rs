use crate::tests::store_fixture;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[tokio::test]
async fn replacing_representation_revokes_prefix_and_rejects_its_old_writer() {
    let (root, store, identity) = store_fixture::mode_fixture("prefix-identity").await;
    let action = store_fixture::open_action_response(&store, &identity, 1).await;
    store
        .write_single_response_for_action(&identity, &action, 0, b"newb")
        .await
        .expect("prefix");
    assert_eq!(
        store.read_range("post", 0..4).await.expect("visible"),
        Some(b"newb".to_vec())
    );
    let replacement = Catalog::new().upsert(
        PostId::new("post"),
        VideoMeta {
            urls: vec!["https://other.example/video".into()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(8),
            duration_ms: Some(1_000),
        },
    );
    store
        .bind_representation(replacement.clone())
        .await
        .expect("replace identity");
    assert_eq!(
        store
            .read_range("post", 0..4)
            .await
            .expect("old prefix withdrawn"),
        None
    );
    assert!(!store
        .write_single_response_for_action(&identity, &action, 4, b"ytes")
        .await
        .expect("stale writer"));
    assert_eq!(
        store
            .media_snapshot("post")
            .await
            .expect("snapshot")
            .binding(),
        Some(&replacement)
    );
    assert_eq!(store.used_bytes().await, 0);
    store.release_action(&action).await;
    store_fixture::discard(&root);
}
