use crate::tests::store_fixture;
use crate::tests::transform_publication_fixture::{files, TransformFixture};
use std::sync::Arc;

#[tokio::test]
async fn an_abandoned_transform_is_recovered_before_cached_metadata_returns() {
    let fixture = TransformFixture::new().await;
    let publication = fixture.publication().await;
    let before = files(&fixture.root);
    let store = Arc::clone(&fixture.store);
    let owner = tokio::spawn(async move {
        store
            .publish_transform_authorized(publication, || {
                panic!("simulated owner failure after staging")
            })
            .await
    });
    assert!(
        owner
            .await
            .expect_err("owner stopped after staging")
            .is_panic(),
        "staging owner failed"
    );

    let snapshot = fixture
        .store
        .media_snapshot("post")
        .await
        .expect("recover before observation");
    assert_eq!(snapshot.total_len(), Some(5));
    assert_eq!(
        fixture
            .store
            .read_range("post", 0..5)
            .await
            .expect("original bytes"),
        Some(b"input".to_vec())
    );
    assert_eq!(
        files(&fixture.root),
        before,
        "abandoned staging must be removed even with warm metadata"
    );
    store_fixture::discard(&fixture.root);
}
