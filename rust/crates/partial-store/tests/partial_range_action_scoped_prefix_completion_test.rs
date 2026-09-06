use crate::partial_range_completion::Completion;
use crate::tests::store_fixture;

#[tokio::test]
async fn prefix_growth_and_eof_preserve_identity_and_charge_each_byte_once() {
    let (root, store, identity) = store_fixture::mode_fixture("prefix-completion").await;
    let action = store_fixture::open_action_response(&store, &identity, 1).await;
    store
        .write_single_response_for_action(&identity, &action, 0, b"newb")
        .await
        .expect("prefix");
    let first = store.media_snapshot("post").await.expect("first snapshot");
    assert_eq!(store.used_bytes().await, 4);
    store
        .write_single_response_for_action(&identity, &action, 4, b"ytes")
        .await
        .expect("suffix");
    let full = store.media_snapshot("post").await.expect("full snapshot");
    assert_eq!(full.revision(), first.revision());
    assert_eq!(full.binding(), first.binding());
    assert_eq!(
        store.read_range("post", 0..8).await.expect("read"),
        Some(b"newbytes".to_vec())
    );
    assert!(!full.is_complete());
    assert!(store.finalize("post", None).await.is_err());
    assert!(store
        .finish_single_response_for_action(&identity, &action, Some(8), true)
        .await
        .expect("EOF"));
    let completed = store
        .media_snapshot("post")
        .await
        .expect("completed snapshot");
    assert!(completed.is_complete());
    assert_eq!(completed.revision(), first.revision());
    assert_eq!(
        store.finalize("post", None).await.expect("finalize"),
        Completion::Unverified
    );
    store.release_action(&action).await;
    assert_eq!(store.used_bytes().await, 8);
    store_fixture::discard(&root);
}
