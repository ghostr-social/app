use super::test_fixture::{assert_rolled_back, interrupted_transaction};

#[tokio::test]
async fn partial_transform_commit_rolls_back_exact_canonical_representation() {
    let (root, input) = interrupted_transaction().await;
    assert_rolled_back(&root, &input).await;
    std::fs::remove_dir_all(root).ok();
}
