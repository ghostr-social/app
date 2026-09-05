use crate::tests::store_fixture;

#[tokio::test]
async fn cancellation_withdraws_transient_bytes_and_planning_coverage() {
    let (root, store, identity) = store_fixture::mode_fixture("transient-cancel").await;
    let action = store
        .reserve_action(&identity, 1, 8)
        .await
        .expect("fixture");
    store
        .open_transient_single_response(&identity, &action, store_fixture::exact_response(8))
        .await
        .expect("fixture");
    store
        .write_single_response_for_action(&identity, &action, 0, b"part")
        .await
        .expect("fixture");
    action.revoke();
    store.release_action(&action).await;
    assert_eq!(store.present_ranges("post").await.expect("fixture"), vec![]);
    assert_eq!(store.read_range("post", 0..4).await.expect("fixture"), None);
    assert!(!store.is_complete("post").await.expect("fixture"));
    store_fixture::discard(&root);
}
