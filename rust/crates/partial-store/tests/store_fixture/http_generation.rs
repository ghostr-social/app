use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::representation::{
    HttpGenerationAuthority, HttpGenerationKey, HttpGenerationLease,
};

pub(in crate::tests) fn http_generation(
    final_url: &str,
    etag: &str,
    epoch: u64,
) -> HttpGenerationAuthority {
    let validator =
        EvidenceValidator::strong_etag(format!("\"{etag}\"")).expect("valid test fixture");
    let key = HttpGenerationKey::try_new(final_url, Some(validator)).expect("valid test fixture");
    HttpGenerationAuthority::Trusted(
        HttpGenerationLease::try_new(key, epoch).expect("valid test fixture"),
    )
}
