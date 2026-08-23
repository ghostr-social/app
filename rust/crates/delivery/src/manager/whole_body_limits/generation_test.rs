use super::WholeBodyLimits;
use ghostr_engine::catalog::{Catalog, HttpObservation, LearnedFacts};
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

const SOURCE: &str = "https://media.example/video.mp4";

#[test]
fn new_http_generation_rearms_an_exhausted_whole_body_cap() {
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("post"), meta());
    let identity = binding.transfer(SOURCE).unwrap();
    assert!(catalog.learn_response_observation_for(&identity, observation("\"v1\"", 1)));
    let mut limits = WholeBodyLimits::default();
    let generation = catalog.http_generation_stamp_for(&identity);
    assert!(limits.record(&catalog, identity.clone(), 8, 13, generation.clone()));
    let exhausted = limits.current(&catalog)[&identity];
    assert_eq!(exhausted.maximum_bytes(), 8);
    assert_eq!(exhausted.observed_bytes(), 13);

    assert!(catalog.learn_response_observation_for(&identity, observation("\"v2\"", 2)));

    assert!(limits.current(&catalog).is_empty());
    assert!(!limits.record(&catalog, identity, 8, 9, generation));
}

fn observation(etag: &str, observed_at_ms: u64) -> HttpObservation {
    HttpObservation::new(
        LearnedFacts::default(),
        Some(SOURCE.to_owned()),
        observed_at_ms,
        EvidenceValidator::strong_etag(etag),
    )
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec![SOURCE.to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: None,
        duration_ms: None,
    }
}
