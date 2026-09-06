use crate::tests::store_fixture;

#[tokio::test]
async fn revoking_an_active_prefix_immediately_hides_bytes_and_releases_them_once() {
    let (root, store, identity) = store_fixture::mode_fixture("prefix-revocation").await;
    let action = store_fixture::open_action_response(&store, &identity, 1).await;
    store
        .write_single_response_for_action(&identity, &action, 0, b"newb")
        .await
        .expect("prefix");
    assert_eq!(
        store.read_range("post", 0..4).await.expect("visible"),
        Some(b"newb".to_vec())
    );
    action.revoke();
    assert_eq!(store.read_range("post", 0..4).await.expect("revoked"), None);
    assert!(store
        .present_ranges("post")
        .await
        .expect("ranges")
        .is_empty());
    assert!(!store
        .write_single_response_for_action(&identity, &action, 4, b"ytes")
        .await
        .expect("late write"));
    store.release_action(&action).await;
    assert_eq!(store.used_bytes().await, 0);
    store.release_action(&action).await;
    assert_eq!(store.used_bytes().await, 0);
    assert!(!store.is_complete("post").await.expect("completion"));
    store_fixture::discard(&root);
}
