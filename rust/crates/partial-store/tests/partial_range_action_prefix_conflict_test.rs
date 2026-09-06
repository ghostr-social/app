use crate::tests::store_fixture;

#[tokio::test]
async fn a_conflicting_probe_keeps_the_whole_prefix_hidden_until_eof() {
    let (root, store, identity) = store_fixture::mode_fixture("prefix-conflict").await;
    store.set_total_len("post", 8).await.expect("known length");
    store
        .write_range("post", 0, b"oldb")
        .await
        .expect("old probe");
    let before = store.media_snapshot("post").await.expect("probe snapshot");
    let action = store_fixture::open_action_response(&store, &identity, 1).await;
    store
        .write_single_response_for_action(&identity, &action, 0, b"newbyt")
        .await
        .expect("conflicting body prefix");
    assert_eq!(
        store
            .read_range("post", 0..4)
            .await
            .expect("read published store state"),
        Some(b"oldb".to_vec())
    );
    assert_eq!(
        store
            .read_range("post", 4..6)
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

    store
        .write_single_response_for_action(&identity, &action, 6, b"es")
        .await
        .expect("remaining body");
    assert_eq!(
        store
            .read_range("post", 0..8)
            .await
            .expect("read published store state"),
        None
    );
    store
        .finish_single_response_for_action(&identity, &action, Some(8), true)
        .await
        .expect("validated EOF");
    assert_eq!(
        store
            .read_range("post", 0..8)
            .await
            .expect("read published store state"),
        Some(b"newbytes".to_vec())
    );
    assert_ne!(
        store
            .media_snapshot("post")
            .await
            .expect("read published store state")
            .revision(),
        before.revision()
    );
    store.release_action(&action).await;
    store_fixture::discard(&root);
}
