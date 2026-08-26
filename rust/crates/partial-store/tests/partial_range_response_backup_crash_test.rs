use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn backed_up_response_crash_restores_canonical_and_discards_stage() {
    let (root, store, binding) =
        crate::tests::store_fixture::staged_replacement("response-backed-up").await;
    crate::tests::store_fixture::backup_canonical(&root).await;
    tokio::fs::write(
        root.join("post.response.commit"),
        crate::tests::store_fixture::response_commit("backed_up"),
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
    assert!(!root.join("post.response.part").exists());
    crate::tests::store_fixture::discard(&root);
}
