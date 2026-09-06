use crate::tests::store_fixture;

#[tokio::test]
async fn active_response_exposes_coherent_prefix_without_claiming_completion() {
    let (root, store, identity) = store_fixture::mode_fixture("action-prefix").await;
    let action = store_fixture::open_action_response(&store, &identity, 1).await;
    assert!(store
        .write_single_response_for_action(&identity, &action, 0, b"newb")
        .await
        .expect("write prefix"));
    assert_eq!(
        store.read_range("post", 0..4).await.expect("read prefix"),
        Some(b"newb".to_vec())
    );
    assert_eq!(
        store
            .read_range("post", 0..8)
            .await
            .expect("read missing suffix"),
        None
    );
    let snapshot = store.media_snapshot("post").await.expect("snapshot");
    assert_eq!(snapshot.total_len(), Some(8));
    assert_eq!(snapshot.ranges().len(), 1);
    assert_eq!(snapshot.ranges()[0], 0..4);
    assert!(snapshot.planning_ranges().is_empty());
    assert!(!snapshot.is_complete());
    assert!(!store.is_complete("post").await.expect("completion"));
    assert!(store.finalize("post", None).await.is_err());
    store.release_action(&action).await;
    store_fixture::discard(&root);
}
