use crate::catalog::{Catalog, CompleteBytesObservation, HttpObservation, LearnedFacts};
use crate::evidence::{EvidenceScope, EvidenceSource, EvidenceTime, EvidenceValidator};
use crate::{DeliveryKind, PostId, VideoMeta};
use core::num::NonZeroU64;

const URL: &str = "https://media.example/video.mp4";

#[test]
fn clean_eof_size_is_bound_to_its_response_generation_and_final_origin() {
    let post = PostId::new("complete-generation");
    let mut catalog = Catalog::new();
    let identity = catalog
        .upsert(post.clone(), metadata())
        .transfer(URL)
        .expect("valid test fixture");
    assert!(catalog.learn_response_observation_for(&identity, response("v1", 1)));
    let v1 = catalog
        .http_generation_for(&identity)
        .expect("valid test fixture");
    assert!(catalog.learn_head_observation_for(&identity, response("v2", 3)));
    let v2 = catalog
        .http_generation_for(&identity)
        .expect("valid test fixture");

    assert!(!catalog.learn_complete_bytes_for(&identity, completed(v1, 4)));
    assert_eq!(
        catalog
            .lookup(&post)
            .expect("valid test fixture")
            .conservative_size_for(URL, 4)
            .exact,
        None
    );
    assert!(catalog.learn_complete_bytes_for(&identity, completed(v2, 5)));

    let entry = catalog.lookup(&post).expect("valid test fixture");
    assert_eq!(entry.conservative_size_for(URL, 5).exact, Some(16));
    assert!(entry.evidence().records().iter().any(|item| {
        matches!(&item.source, EvidenceSource::CompleteBytes { origin } if origin == "cdn.example")
            && matches!(&item.scope, EvidenceScope::ValidatedUrl { validator, .. }
                if validator == &etag("v2"))
    }));
}

fn response(version: &str, order: u64) -> HttpObservation {
    HttpObservation::new(
        LearnedFacts::default(),
        None,
        EvidenceTime::ordered(100, order),
        Some(etag(version)),
    )
    .with_final_url("https://cdn.example/final.mp4")
}

fn completed(
    generation: crate::representation::HttpGenerationLease,
    order: u64,
) -> CompleteBytesObservation {
    CompleteBytesObservation::new(
        NonZeroU64::new(16).expect("valid test fixture"),
        "https://cdn.example/final.mp4",
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
        urls: vec![URL.into()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: None,
        duration_ms: None,
    }
}
