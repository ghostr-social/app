use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn corrupt_committed_response_rolls_back_to_prior_canonical() {
    let (root, store, binding) =
        crate::tests::store_fixture::staged_replacement("response-corrupt-commit").await;
    crate::tests::store_fixture::backup_canonical(&root).await;
    tokio::fs::rename(root.join("post.response.part"), root.join("post.video"))
        .await
        .expect("valid test fixture");
    tokio::fs::rename(
        root.join("post.response.ranges"),
        root.join("post.ranges.json"),
    )
    .await
    .expect("valid test fixture");
    tokio::fs::write(root.join("post.video"), b"badbytes")
        .await
        .expect("valid test fixture");
    tokio::fs::write(root.join("post.verified"), b"")
        .await
        .expect("valid test fixture");
    tokio::fs::write(
        root.join("post.response.commit"),
        crate::tests::store_fixture::response_commit("committed"),
    )
    .await
    .expect("valid test fixture");
    drop(store);

    let reopened = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.expect("valid test fixture");
    reopened
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    assert_eq!(
        reopened
            .read_range("post", 0..8)
            .await
            .expect("valid test fixture"),
        Some(b"oldbytes".to_vec())
    );
    crate::tests::store_fixture::discard(&root);
}
