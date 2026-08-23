use ghostr_engine::catalog::{Catalog, CompleteBytesObservation, HttpObservation, LearnedFacts};
use ghostr_engine::evidence::{EvidenceTime, EvidenceValidator};
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::num::NonZeroU64;

const SOURCE: &str = "https://media.example/video.mp4";
const FINAL: &str = "https://cdn.example/video.mp4";

#[test]
fn validator_aba_cannot_authorize_an_old_response_completion() {
    let post = PostId::new("http-generation-aba");
    let mut catalog = Catalog::new();
    let identity = catalog
        .upsert(post.clone(), metadata())
        .transfer(SOURCE)
        .unwrap();
    assert!(catalog.learn_response_observation_for(&identity, response("v1", 1)));
    let old_v1 = catalog.http_generation_for(&identity).unwrap();
    assert!(catalog.learn_response_observation_for(&identity, response("v2", 2)));
    assert!(catalog.learn_response_observation_for(&identity, response("v1", 3)));
    let current_v1 = catalog.http_generation_for(&identity).unwrap();

    assert_ne!(old_v1, current_v1);
    assert!(!catalog.learn_complete_bytes_for(&identity, completed(old_v1, 4)));
    assert!(catalog.learn_complete_bytes_for(&identity, completed(current_v1, 5)));
    assert_eq!(
        catalog
            .lookup(&post)
            .unwrap()
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
    generation: ghostr_engine::representation::HttpGenerationLease,
    order: u64,
) -> CompleteBytesObservation {
    CompleteBytesObservation::new(
        NonZeroU64::new(16).unwrap(),
        FINAL,
        EvidenceTime::ordered(100, order),
        generation.key().validator().cloned(),
    )
    .with_generation(generation)
}

fn etag(value: &str) -> EvidenceValidator {
    EvidenceValidator::strong_etag(format!("\"{value}\"")).unwrap()
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
