use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::representation::{
    HttpGenerationAuthority, HttpGenerationKey, HttpGenerationLease,
};

pub fn http_generation(final_url: &str, etag: &str, epoch: u64) -> HttpGenerationAuthority {
    let validator = EvidenceValidator::strong_etag(format!("\"{etag}\"")).unwrap();
    let key = HttpGenerationKey::try_new(final_url, Some(validator)).unwrap();
    HttpGenerationAuthority::Trusted(HttpGenerationLease::try_new(key, epoch).unwrap())
}
