use ghostr_delivery::delivery_events::FocusItem;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::representation::{
    HttpGenerationAuthority, HttpGenerationKey, HttpGenerationLease,
};
use ghostr_partial_store::partial_range_store::PartialRangeStore;

pub async fn seed_probe(store: &PartialRangeStore, item: &FocusItem) {
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(item.post.clone(), item.meta.clone());
    let source = item.meta.urls.first().expect("fixture source");
    let identity = binding.transfer(source).expect("fixture identity");
    store
        .bind_representation(binding)
        .await
        .expect("bind probe");
    store
        .select_transfer(identity.clone())
        .await
        .expect("select probe");
    store
        .apply_http_generation(&identity, authority(source))
        .await
        .expect("probe authority");
    store
        .set_total_len(item.post.as_str(), super::continuous_body_fixture::TOTAL)
        .await
        .expect("probe length");
    store
        .write_range(item.post.as_str(), 0, &vec![7; 65536])
        .await
        .expect("probe bytes");
}

fn authority(source: &str) -> HttpGenerationAuthority {
    let validator =
        EvidenceValidator::last_modified("Thu, 03 Sep 2026 23:47:55 GMT").expect("date");
    let key = HttpGenerationKey::try_new(source, Some(validator)).expect("key");
    HttpGenerationAuthority::Trusted(HttpGenerationLease::try_new(key, 1).expect("lease"))
}
