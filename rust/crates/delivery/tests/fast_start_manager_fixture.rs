use ghostr_delivery::delivery_events::{FocusItem, PlayerPreparationAuthority};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_partial_store::partial_range_store::{ContentRevision, PartialRangeStore};

pub async fn seed(
    store: &PartialRangeStore,
    item: &FocusItem,
    bytes: &[u8],
) -> RepresentationBinding {
    let binding = Catalog::new().upsert(item.post.clone(), item.meta.clone());
    let key = item.post.as_str();
    store.bind_representation(binding.clone()).await.unwrap();
    store.set_total_len(key, bytes.len() as u64).await.unwrap();
    store.write_range(key, 0, bytes).await.unwrap();
    store.finalize(key, None).await.unwrap();
    binding
}

pub fn authority(
    binding: RepresentationBinding,
    revision: ContentRevision,
) -> PlayerPreparationAuthority {
    PlayerPreparationAuthority::try_new(binding.post().clone(), binding, revision).unwrap()
}
