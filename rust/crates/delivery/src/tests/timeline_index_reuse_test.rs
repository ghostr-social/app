use crate::manager::timeline::TimelineCoordinator;
use crate::tests::{
    media_timeline_fixture::classic_moov, support::temp_directory,
    timeline_index_fixture as fixture,
};

#[tokio::test]
async fn fresh_public_source_reuses_compiled_structure_after_payload_eviction() {
    let root = temp_directory("timeline-index-reuse");
    let (store, binding) = fixture::store(&root).await;
    let mut coordinator = TimelineCoordinator::new(std::sync::Arc::clone(&store));
    fixture::observe(&store, &binding, &mut coordinator, "\"v1\"").await;
    store
        .write_range("post", 0, &classic_moov(800, 100))
        .await
        .expect("fixture");
    let expected = fixture::run(&store, &binding, &mut coordinator)
        .await
        .expect("cold parsed index");

    store.quarantine("post").await.expect("fixture");
    drop(coordinator);
    drop(store);
    let (store, binding) = fixture::store(&root).await;
    store.load_existing().await.expect("fixture");
    let mut coordinator = TimelineCoordinator::new(std::sync::Arc::clone(&store));
    fixture::observe(&store, &binding, &mut coordinator, "\"v1\"").await;
    store
        .write_range("post", 0, b"\0\0\0\x08free")
        .await
        .expect("fixture");
    assert_eq!(
        fixture::run(&store, &binding, &mut coordinator).await,
        Some(expected)
    );
    assert_eq!(
        store.present_ranges("post").await.expect("fixture"),
        vec![0..8]
    );

    let mut changed = TimelineCoordinator::new(std::sync::Arc::clone(&store));
    fixture::observe(&store, &binding, &mut changed, "\"v2\"").await;
    store
        .write_range("post", 0, b"\0\0\0\x08free")
        .await
        .expect("fixture");
    assert_eq!(fixture::run(&store, &binding, &mut changed).await, None);
    tokio::fs::remove_dir_all(root).await.expect("fixture");
}
