mod store_fixture;

use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn backed_up_response_crash_restores_canonical_and_discards_stage() {
    let (root, store, binding) = store_fixture::staged_replacement("response-backed-up").await;
    store_fixture::backup_canonical(&root).await;
    tokio::fs::write(
        root.join("post.response.commit"),
        store_fixture::response_commit("backed_up"),
    )
    .await
    .unwrap();
    drop(store);

    let reopened = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.unwrap();
    reopened.bind_representation(binding).await.unwrap();
    assert_eq!(
        reopened.read_range("post", 0..8).await.unwrap(),
        Some(b"oldbytes".to_vec())
    );
    assert!(!root.join("post.response.part").exists());
    store_fixture::discard(&root);
}
