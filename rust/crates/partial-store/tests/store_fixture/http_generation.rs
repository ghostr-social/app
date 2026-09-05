use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::representation::{
    HttpGenerationAuthority, HttpGenerationKey, HttpGenerationLease,
};

pub(in crate::tests) async fn authorize(
    store: &crate::partial_range_store::PartialRangeStore,
    identity: &ghostr_engine::representation::TransferIdentity,
    etag: &str,
) {
    let authority = http_generation(identity.source().as_str(), etag, 1);
    assert!(store
        .apply_http_generation(identity, authority)
        .await
        .expect("HTTP authority"));
}

pub(in crate::tests) fn http_generation(
    final_url: &str,
    etag: &str,
    epoch: u64,
) -> HttpGenerationAuthority {
    let validator = EvidenceValidator::strong_etag(format!("\"{etag}\"")).expect("fixture");
    let key = HttpGenerationKey::try_new(final_url, Some(validator)).expect("fixture");
    HttpGenerationAuthority::Trusted(HttpGenerationLease::try_new(key, epoch).expect("fixture"))
}
