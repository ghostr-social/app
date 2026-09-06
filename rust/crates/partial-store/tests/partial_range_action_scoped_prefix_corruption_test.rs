use crate::tests::store_fixture;

#[tokio::test]
async fn corrupted_active_prefix_is_withdrawn_and_cannot_continue_writing() {
    let (root, store, identity) = store_fixture::mode_fixture("prefix-corruption").await;
    let action = store_fixture::open_action_response(&store, &identity, 1).await;
    store
        .write_single_response_for_action(&identity, &action, 0, b"newb")
        .await
        .expect("prefix");
    assert_eq!(
        store.read_range("post", 0..4).await.expect("original"),
        Some(b"newb".to_vec())
    );
    tokio::fs::write(root.join("post.response.part"), b"badb")
        .await
        .expect("external corruption");
    assert_eq!(
        store
            .read_range("post", 0..4)
            .await
            .expect("corruption check"),
        None
    );
    assert_eq!(store.used_bytes().await, 0);
    assert!(!store
        .write_single_response_for_action(&identity, &action, 4, b"ytes")
        .await
        .expect("late write"));
    store.release_action(&action).await;
    assert_eq!(store.used_bytes().await, 0);
    store_fixture::discard(&root);
}
