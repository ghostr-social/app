#[path = "staged_commit_reload_fixture.rs"]
mod staged_commit_reload_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{ByteRange, PostId};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn restart_keeps_a_published_stage_when_sparse_retirement_was_interrupted() {
    let root = crate::tests::store_fixture::temp_root("staged-commit-reload");
    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), staged_commit_reload_fixture::meta());
    let sparse_identity = binding
        .transfer("https://a.example/video")
        .expect("valid test fixture");
    let whole_identity = sparse_identity.clone();
    let generation = SourceGeneration::try_new(sparse_identity.source().as_str(), "\"a\"", 8)
        .expect("valid test fixture");
    store
        .bind_representation(binding.clone())
        .await
        .expect("valid test fixture");
    store
        .apply_http_generation(
            &sparse_identity,
            crate::tests::store_fixture::http_generation("https://a.example/video", "a", 1),
        )
        .await
        .expect("valid test fixture");
    store
        .select_transfer(sparse_identity.clone())
        .await
        .expect("valid test fixture");
    let sparse = store
        .reserve_action(&sparse_identity, 1, 8)
        .await
        .expect("valid test fixture");
    store
        .open_sparse_response(
            &sparse_identity,
            &sparse,
            generation.clone(),
            ByteRange::new(0, 8),
        )
        .await
        .expect("valid test fixture");
    store
        .write_range_for_action_if_current(&sparse_identity, &generation, &sparse, 0, b"old!")
        .await
        .expect("valid test fixture");
    let whole = store
        .reserve_action(&whole_identity, 2, 8)
        .await
        .expect("valid test fixture");
    store
        .open_single_response_for_action(
            &whole_identity,
            &whole,
            crate::tests::store_fixture::exact_response(8),
        )
        .await
        .expect("valid test fixture");
    store
        .write_single_response_for_action(&whole_identity, &whole, 0, b"new data")
        .await
        .expect("valid test fixture");
    tokio::fs::create_dir(root.join("post.sparse.intent.tmp"))
        .await
        .expect("valid test fixture");
    assert!(store
        .finish_single_response_for_action(&whole_identity, &whole, Some(8), true)
        .await
        .expect("valid test fixture"));
    assert_eq!(
        store
            .read_range("post", 0..8)
            .await
            .expect("valid test fixture"),
        Some(b"new data".to_vec())
    );
    assert!(
        tokio::fs::metadata(root.join("post.response.commit"))
            .await
            .expect("valid test fixture")
            .len()
            > 0
    );
    drop(store);

    let reopened = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.expect("valid test fixture");
    reopened
        .bind_representation(binding.clone())
        .await
        .expect("valid test fixture");

    assert_eq!(
        reopened
            .read_range("post", 0..8)
            .await
            .expect("valid test fixture"),
        Some(b"new data".to_vec())
    );
    assert!(root.join("post.response.commit").exists());
    let recovered_usage = reopened.used_bytes().await;
    assert!((8..=16).contains(&recovered_usage));
    tokio::fs::remove_dir(root.join("post.sparse.intent.tmp"))
        .await
        .expect("valid test fixture");
    let retry = reopened
        .reserve_action(&whole_identity, 3, 8)
        .await
        .expect("valid test fixture");
    assert_eq!(reopened.used_bytes().await, 8);
    reopened.release_action(&retry).await;
    assert!(!root.join("post.response.commit").exists());
    assert!(!root.join("post.sparse.intent").exists());
    crate::tests::store_fixture::discard(&root);
}
