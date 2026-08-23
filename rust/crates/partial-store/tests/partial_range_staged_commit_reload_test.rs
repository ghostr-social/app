mod store_fixture;

#[path = "staged_commit_reload_fixture.rs"]
mod staged_commit_reload_fixture;

use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::{ByteRange, PostId};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn restart_keeps_a_published_stage_when_sparse_retirement_was_interrupted() {
    let root = store_fixture::temp_root("staged-commit-reload");
    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), staged_commit_reload_fixture::meta());
    let sparse_identity = binding.transfer("https://a.example/video").unwrap();
    let whole_identity = sparse_identity.clone();
    let generation =
        SourceGeneration::try_new(sparse_identity.source().as_str(), "\"a\"", 8).unwrap();
    store.bind_representation(binding.clone()).await.unwrap();
    store
        .apply_http_generation(
            &sparse_identity,
            store_fixture::http_generation("https://a.example/video", "a", 1),
        )
        .await
        .unwrap();
    store
        .select_transfer(sparse_identity.clone())
        .await
        .unwrap();
    let sparse = store.reserve_action(&sparse_identity, 1, 8).await.unwrap();
    store
        .open_sparse_response(
            &sparse_identity,
            &sparse,
            generation.clone(),
            ByteRange::new(0, 8),
        )
        .await
        .unwrap();
    store
        .write_range_for_action_if_current(&sparse_identity, &generation, &sparse, 0, b"old!")
        .await
        .unwrap();
    let whole = store.reserve_action(&whole_identity, 2, 8).await.unwrap();
    store
        .open_single_response_for_action(&whole_identity, &whole, store_fixture::exact_response(8))
        .await
        .unwrap();
    store
        .write_single_response_for_action(&whole_identity, &whole, 0, b"new data")
        .await
        .unwrap();
    tokio::fs::create_dir(root.join("post.sparse.intent.tmp"))
        .await
        .unwrap();
    assert!(store
        .finish_single_response_for_action(&whole_identity, &whole, Some(8), true)
        .await
        .unwrap());
    assert_eq!(
        store.read_range("post", 0..8).await.unwrap(),
        Some(b"new data".to_vec())
    );
    assert!(
        tokio::fs::metadata(root.join("post.response.commit"))
            .await
            .unwrap()
            .len()
            > 0
    );
    drop(store);

    let reopened = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.unwrap();
    reopened.bind_representation(binding.clone()).await.unwrap();

    assert_eq!(
        reopened.read_range("post", 0..8).await.unwrap(),
        Some(b"new data".to_vec())
    );
    assert!(root.join("post.response.commit").exists());
    let recovered_usage = reopened.used_bytes().await;
    assert!((8..=16).contains(&recovered_usage));
    tokio::fs::remove_dir(root.join("post.sparse.intent.tmp"))
        .await
        .unwrap();
    let retry = reopened
        .reserve_action(&whole_identity, 3, 8)
        .await
        .unwrap();
    assert_eq!(reopened.used_bytes().await, 8);
    reopened.release_action(&retry).await;
    assert!(!root.join("post.response.commit").exists());
    assert!(!root.join("post.sparse.intent").exists());
    store_fixture::discard(&root);
}
