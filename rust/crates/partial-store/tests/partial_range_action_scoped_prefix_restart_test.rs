use crate::tests::store_fixture;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn interrupted_prefix_is_not_reused_or_accounted_after_restart() {
    let (root, store, identity) = store_fixture::mode_fixture("prefix-restart").await;
    let action = store_fixture::open_action_response(&store, &identity, 1).await;
    store
        .write_single_response_for_action(&identity, &action, 0, b"newb")
        .await
        .expect("prefix");
    let snapshot = store.media_snapshot("post").await.expect("snapshot");
    assert_eq!(
        store.read_range("post", 0..4).await.expect("visible"),
        Some(b"newb".to_vec())
    );
    drop(store);
    drop(action);
    let reopened = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.expect("reload");
    reopened
        .bind_representation(snapshot.binding().expect("binding").clone())
        .await
        .expect("rebind");
    assert_eq!(
        reopened
            .read_range("post", 0..4)
            .await
            .expect("restart read"),
        None
    );
    assert_eq!(reopened.used_bytes().await, 0);
    assert!(!reopened.is_complete("post").await.expect("completion"));
    store_fixture::discard(&root);
}
