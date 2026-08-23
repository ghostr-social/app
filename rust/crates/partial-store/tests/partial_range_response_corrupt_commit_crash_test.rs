mod store_fixture;

use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn corrupt_committed_response_rolls_back_to_prior_canonical() {
    let (root, store, binding) = store_fixture::staged_replacement("response-corrupt-commit").await;
    store_fixture::backup_canonical(&root).await;
    tokio::fs::rename(root.join("post.response.part"), root.join("post.video"))
        .await
        .unwrap();
    tokio::fs::rename(
        root.join("post.response.ranges"),
        root.join("post.ranges.json"),
    )
    .await
    .unwrap();
    tokio::fs::write(root.join("post.video"), b"badbytes")
        .await
        .unwrap();
    tokio::fs::write(root.join("post.verified"), b"")
        .await
        .unwrap();
    tokio::fs::write(
        root.join("post.response.commit"),
        store_fixture::response_commit("committed"),
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
    store_fixture::discard(&root);
}
