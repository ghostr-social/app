use crate::catalog::{Catalog, CompleteBytesObservation, HttpObservation, LearnedFacts};
use crate::evidence::{EvidenceTime, EvidenceValidator};
use crate::{DeliveryKind, PostId, VideoMeta};
use core::num::NonZeroU64;

const SOURCE: &str = "https://media.example/video.mp4";
const FINAL: &str = "https://cdn.example/video.mp4";

#[test]
fn validator_aba_cannot_authorize_an_old_response_completion() {
    let post = PostId::new("http-generation-aba");
    let mut catalog = Catalog::new();
    let identity = catalog
        .upsert(post.clone(), metadata())
        .transfer(SOURCE)
        .expect("valid test fixture");
    assert!(catalog.learn_response_observation_for(&identity, response("v1", 1)));
    let old_v1 = catalog
        .http_generation_for(&identity)
        .expect("valid test fixture");
    assert!(catalog.learn_response_observation_for(&identity, response("v2", 2)));
    assert!(catalog.learn_response_observation_for(&identity, response("v1", 3)));
    let current_v1 = catalog
        .http_generation_for(&identity)
        .expect("valid test fixture");

    assert_ne!(old_v1, current_v1);
    assert!(!catalog.learn_complete_bytes_for(&identity, completed(old_v1, 4)));
    assert!(catalog.learn_complete_bytes_for(&identity, completed(current_v1, 5)));
    assert_eq!(
        catalog
            .lookup(&post)
            .expect("valid test fixture")
            .conservative_size_for(SOURCE, 5)
            .exact,
        Some(16)
    );
}

fn response(version: &str, order: u64) -> HttpObservation {
    HttpObservation::new(
        LearnedFacts::default(),
        None,
        EvidenceTime::ordered(100, order),
        Some(etag(version)),
    )
    .with_final_url(FINAL)
}

fn completed(
    generation: crate::representation::HttpGenerationLease,
    order: u64,
) -> CompleteBytesObservation {
    CompleteBytesObservation::new(
        NonZeroU64::new(16).expect("valid test fixture"),
        FINAL,
        EvidenceTime::ordered(100, order),
        generation.key().validator().cloned(),
    )
    .with_generation(generation)
}

fn etag(value: &str) -> EvidenceValidator {
    EvidenceValidator::strong_etag(format!("\"{value}\"")).expect("valid test fixture")
}

fn metadata() -> VideoMeta {
    VideoMeta {
        urls: vec![SOURCE.into()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: None,
        duration_ms: None,
    }
}
