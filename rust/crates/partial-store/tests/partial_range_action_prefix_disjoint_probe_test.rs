use crate::tests::store_fixture;

#[tokio::test]
async fn matching_the_head_does_not_prove_a_disjoint_tail() {
    let (root, store, identity) = store_fixture::mode_fixture("prefix-disjoint-probe").await;
    store.set_total_len("post", 8).await.expect("known length");
    store
        .write_range("post", 0, b"ne")
        .await
        .expect("head probe");
    store
        .write_range("post", 6, b"ZZ")
        .await
        .expect("tail probe");
    let before = store.media_snapshot("post").await.expect("probe snapshot");
    let action = store_fixture::open_action_response(&store, &identity, 1).await;
    store
        .write_single_response_for_action(&identity, &action, 0, b"newbyt")
        .await
        .expect("matching head");
    assert_eq!(
        store
            .read_range("post", 2..6)
            .await
            .expect("read published store state"),
        None
    );
    assert_eq!(
        store
            .read_range("post", 6..8)
            .await
            .expect("read published store state"),
        Some(b"ZZ".to_vec())
    );
    store
        .write_single_response_for_action(&identity, &action, 6, b"es")
        .await
        .expect("conflicting tail");
    assert_eq!(
        store
            .read_range("post", 0..8)
            .await
            .expect("read published store state"),
        None
    );
    assert_eq!(
        store
            .media_snapshot("post")
            .await
            .expect("read published store state")
            .revision(),
        before.revision()
    );
    action.revoke();
    store.release_action(&action).await;
    assert_eq!(store.used_bytes().await, 4);
    assert_eq!(
        store
            .media_snapshot("post")
            .await
            .expect("read published store state")
            .revision(),
        before.revision()
    );
    store_fixture::discard(&root);
}
