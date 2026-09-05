use crate::tests::store_fixture;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn no_store_response_is_readable_in_memory_but_never_persisted_even_after_hash_verification()
{
    let (root, store, identity) = store_fixture::mode_fixture("transient-response").await;
    let action = store
        .reserve_action(&identity, 1, 8)
        .await
        .expect("fixture");
    store
        .open_transient_single_response(&identity, &action, store_fixture::exact_response(8))
        .await
        .expect("fixture");
    store
        .write_single_response_for_action(&identity, &action, 0, b"new ")
        .await
        .expect("fixture");
    assert_eq!(
        store.read_range("post", 0..4).await.expect("fixture"),
        Some(b"new ".to_vec())
    );
    assert_eq!(
        store
            .media_snapshot("post")
            .await
            .expect("fixture")
            .planning_ranges(),
        &[]
    );
    store
        .write_single_response_for_action(&identity, &action, 4, b"data")
        .await
        .expect("fixture");
    assert!(store
        .finish_single_response_for_action(&identity, &action, Some(8), true)
        .await
        .expect("fixture"));
    store.release_action(&action).await;
    let digest = "d5b7f828235a92d3d280fa08f3ddb9e5b6947123b44091c92db7594aa1408614";
    assert!(store.finalize("post", Some(digest)).await.is_ok());
    assert_eq!(
        store.read_range("post", 0..8).await.expect("fixture"),
        Some(b"new data".to_vec())
    );
    assert_eq!(store.used_bytes().await, 0);
    assert!(!root.join("post.part").exists());
    assert!(!root.join("post.mp4").exists());
    let reopened = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.expect("fixture");
    assert_eq!(
        reopened.read_range("post", 0..8).await.expect("fixture"),
        None
    );
    store.clear().await.expect("fixture");
    assert_eq!(store.read_range("post", 0..8).await.expect("fixture"), None);
    store_fixture::discard(&root);
}
