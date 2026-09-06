use crate::tests::store_fixture;

#[tokio::test]
async fn a_whole_response_extends_its_matching_probe_prefix_before_eof() {
    let (root, store, identity) = store_fixture::mode_fixture("prefix-existing-head").await;
    store.set_total_len("post", 8).await.expect("known length");
    store
        .write_range("post", 0, b"newb")
        .await
        .expect("probe bytes");
    let before = store.media_snapshot("post").await.expect("probe snapshot");
    let action = store_fixture::open_action_response(&store, &identity, 1).await;
    store
        .write_single_response_for_action(&identity, &action, 0, b"ne")
        .await
        .expect("first body fragment");
    assert_eq!(
        store
            .read_range("post", 0..4)
            .await
            .expect("probe stays readable"),
        Some(b"newb".to_vec())
    );
    store
        .write_single_response_for_action(&identity, &action, 2, b"wbyt")
        .await
        .expect("body passes probe");

    assert_eq!(
        store
            .read_range("post", 0..6)
            .await
            .expect("stream before EOF"),
        Some(b"newbyt".to_vec())
    );
    let after = store.media_snapshot("post").await.expect("stream snapshot");
    assert_eq!(
        after.revision(),
        before.revision(),
        "identical previously served bytes must not reset the player"
    );
    assert!(
        after.planning_ranges().is_empty(),
        "one response grants no resumable byte authority"
    );
    assert!(!after.is_complete(), "the response has not reached EOF");
    assert_eq!(
        store.used_bytes().await,
        10,
        "probe and staged response are both accounted"
    );
    action.revoke();
    store.release_action(&action).await;
    assert_eq!(
        store
            .read_range("post", 0..4)
            .await
            .expect("retained probe"),
        Some(b"newb".to_vec())
    );
    assert_eq!(
        store
            .read_range("post", 4..6)
            .await
            .expect("revoked suffix"),
        None
    );
    assert_eq!(
        store.used_bytes().await,
        4,
        "only the probe survives cancellation"
    );
    store_fixture::discard(&root);
}
