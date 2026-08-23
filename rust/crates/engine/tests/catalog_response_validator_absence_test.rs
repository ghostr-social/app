use ghostr_engine::catalog::{Catalog, HttpObservation, LearnedFacts};
use ghostr_engine::evidence::{EvidenceTime, EvidenceValidator};
use ghostr_engine::representation::HttpGenerationAuthority;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

const URL: &str = "https://media.example/video.mp4";

#[test]
fn response_validator_absence_cannot_claim_or_revoke_trusted_generation() {
    let post = PostId::new("validator-absence");
    let mut catalog = Catalog::new();
    let identity = catalog
        .upsert(post.clone(), metadata())
        .transfer(URL)
        .unwrap();
    assert!(catalog.learn_response_observation_for(
        &identity,
        observation(Some(16), EvidenceValidator::strong_etag("\"v1\""), 1)
    ));
    let original = catalog.http_generation_for(&identity).unwrap();
    assert!(catalog.learn_head_observation_for(&identity, observation(None, None, 2)));
    assert_eq!(
        catalog.http_generation_for(&identity),
        Some(original.clone())
    );

    assert!(!catalog.learn_response_observation_for(&identity, observation(None, None, 3)));
    let rejected = catalog
        .reject_response_generation_for(
            &identity,
            "https://redirect.example/video.mp4",
            None,
            EvidenceTime::ordered(100, 4),
        )
        .unwrap();
    assert_eq!(rejected, HttpGenerationAuthority::Trusted(original.clone()));
    assert_eq!(catalog.http_generation_for(&identity), Some(original));
    let entry = catalog.lookup(&post).unwrap();
    let expected = EvidenceValidator::strong_etag("\"v1\"");
    assert_eq!(entry.current_validator_for(URL), expected.as_ref());
    assert_eq!(entry.conservative_size_for(URL, 4).exact, Some(16));
}

fn observation(
    size: Option<u64>,
    validator: Option<EvidenceValidator>,
    order: u64,
) -> HttpObservation {
    HttpObservation::new(
        LearnedFacts {
            content_length: size,
            ..LearnedFacts::default()
        },
        None,
        EvidenceTime::ordered(100, order),
        validator,
    )
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
