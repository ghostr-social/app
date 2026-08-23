use ghostr_engine::catalog::{Catalog, CompleteBytesObservation, HttpObservation, LearnedFacts};
use ghostr_engine::evidence::{EvidenceTime, EvidenceValidator};
use ghostr_engine::representation::HttpGenerationAuthority;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::num::NonZeroU64;

const SOURCE: &str = "https://media.example/video.mp4";
const FINAL: &str = "https://cdn.example/video.mp4";

#[test]
fn changed_validator_on_malformed_media_revokes_old_authority_without_new_facts() {
    let post = PostId::new("unusable-generation");
    let mut catalog = Catalog::new();
    let identity = catalog
        .upsert(post.clone(), metadata())
        .transfer(SOURCE)
        .unwrap();
    assert!(catalog.learn_response_observation_for(&identity, accepted()));
    let old = catalog.http_generation_for(&identity).unwrap();

    let authority = catalog
        .reject_response_generation_for(
            &identity,
            FINAL,
            Some(etag("v2")),
            EvidenceTime::ordered(100, 2),
        )
        .unwrap();

    assert!(matches!(authority, HttpGenerationAuthority::Unknown(_)));
    assert_eq!(catalog.http_generation_for(&identity), None);
    let entry = catalog.lookup(&post).unwrap();
    assert_eq!(entry.current_validator_for(SOURCE), None);
    assert_eq!(entry.authoritative_total_for(SOURCE), None);
    assert_eq!(entry.observed_range_support_for(SOURCE), None);
    let stale = CompleteBytesObservation::new(
        NonZeroU64::new(16).unwrap(),
        FINAL,
        EvidenceTime::ordered(100, 3),
        Some(etag("v1")),
    )
    .with_generation(old);
    assert!(!catalog.learn_complete_bytes_for(&identity, stale));
}

fn accepted() -> HttpObservation {
    HttpObservation::new(
        LearnedFacts {
            content_length: Some(16),
            accept_ranges: Some(true),
            host: Some("cdn.example".into()),
        },
        Some("video/mp4".into()),
        EvidenceTime::ordered(100, 1),
        Some(etag("v1")),
    )
    .with_final_url(FINAL)
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
