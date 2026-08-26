mod delivery_fixture;

use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::wait::wait_cache_first;
use delivery_fixture::{start_harness_with_store, temp_directory};
use ghostr_delivery::cache_registry::CacheStatus;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::SourceGeneration;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn cache_status_uses_the_authoritative_stored_extent() {
    let root = temp_directory("cache-canonical-extent");
    let store = Arc::new(PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    ));
    let mut item = sized_item("post", "https://primary.example/video.mp4", 8, 1_000);
    item.meta
        .urls
        .push("https://mirror.example/video.mp4".to_owned());
    seed_mirror_prefix(&store, &item).await;
    let harness = start_harness_with_store(store, root, DeliveryOptions::default());

    harness.handle.update_focus(focus_now(vec![item], 0, 0));
    wait_cache_first(&harness.cache, "post").await;

    assert_eq!(harness.cache.videos()[0].status, CacheStatus::Partial);
    std::fs::remove_dir_all(harness.root).ok();
}

async fn seed_mirror_prefix(
    store: &PartialRangeStore,
    item: &ghostr_delivery::delivery_events::FocusItem,
) {
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(item.post.clone(), item.meta.clone());
    let mirror = &item.meta.urls[1];
    let identity = binding.transfer(mirror).expect("valid test fixture");
    let generation =
        SourceGeneration::try_new(mirror, "\"mirror\"", 16).expect("valid test fixture");
    store
        .bind_representation(binding)
        .await
        .expect("valid test fixture");
    store
        .select_transfer(identity.clone())
        .await
        .expect("valid test fixture");
    store
        .accept_generation(&identity, generation)
        .await
        .expect("valid test fixture");
    store
        .set_total_len("post", 16)
        .await
        .expect("valid test fixture");
    store
        .write_range("post", 0, b"01234567")
        .await
        .expect("valid test fixture");
}
