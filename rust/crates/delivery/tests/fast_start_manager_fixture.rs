use ghostr_delivery::delivery_events::{FocusItem, PlayerPreparationAuthority};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::{ContentRevision, PartialRangeStore};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn new_store(root: &Path) -> Arc<PartialRangeStore> {
    Arc::new(PartialRangeStore::with_capacity(
        root.to_path_buf(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    ))
}

pub fn bindings(items: [&FocusItem; 2]) -> [RepresentationBinding; 2] {
    let mut catalog = Catalog::new();
    items.map(|item| catalog.upsert(item.post.clone(), item.meta.clone()))
}

pub async fn seed(store: &PartialRangeStore, binding: RepresentationBinding, bytes: &[u8]) {
    let key = binding.post().as_str();
    store
        .bind_representation(binding.clone())
        .await
        .expect("valid test fixture");
    store
        .set_total_len(key, bytes.len() as u64)
        .await
        .expect("valid test fixture");
    store
        .write_range(key, 0, bytes)
        .await
        .expect("valid test fixture");
    store.finalize(key, None).await.expect("valid test fixture");
}

pub fn authority(
    binding: RepresentationBinding,
    revision: ContentRevision,
) -> PlayerPreparationAuthority {
    PlayerPreparationAuthority::try_new(binding.post().clone(), binding, revision, "asset")
        .expect("valid test fixture")
}
