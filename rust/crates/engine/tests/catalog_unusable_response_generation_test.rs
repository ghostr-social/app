use crate::catalog::{Catalog, CompleteBytesObservation, HttpObservation, LearnedFacts};
use crate::evidence::{EvidenceTime, EvidenceValidator};
use crate::representation::HttpGenerationAuthority;
use crate::{DeliveryKind, PostId, VideoMeta};
use core::num::NonZeroU64;

const SOURCE: &str = "https://media.example/video.mp4";
const FINAL: &str = "https://cdn.example/video.mp4";

#[test]
fn changed_validator_on_malformed_media_revokes_old_authority_without_new_facts() {
    let post = PostId::new("unusable-generation");
    let mut catalog = Catalog::new();
    let identity = catalog
        .upsert(post.clone(), metadata())
        .transfer(SOURCE)
        .expect("valid test fixture");
    assert!(catalog.learn_response_observation_for(&identity, accepted()));
    let old = catalog
        .http_generation_for(&identity)
        .expect("valid test fixture");

    let authority = catalog
        .reject_response_generation_for(
            &identity,
            FINAL,
            Some(etag("v2")),
            EvidenceTime::ordered(100, 2),
        )
        .expect("valid test fixture");

    assert!(matches!(authority, HttpGenerationAuthority::Unknown(_)));
    assert_eq!(catalog.http_generation_for(&identity), None);
    let entry = catalog.lookup(&post).expect("valid test fixture");
    assert_eq!(entry.current_validator_for(SOURCE), None);
    assert_eq!(entry.authoritative_total_for(SOURCE), None);
    assert_eq!(entry.observed_range_support_for(SOURCE), None);
    let stale = CompleteBytesObservation::new(
        NonZeroU64::new(16).expect("valid test fixture"),
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
