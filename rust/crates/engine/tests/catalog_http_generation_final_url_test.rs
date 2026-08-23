use ghostr_engine::catalog::{Catalog, HttpObservation, LearnedFacts};
use ghostr_engine::evidence::{EvidenceTime, EvidenceValidator};
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

const SOURCE: &str = "https://media.example/video.mp4";

#[test]
fn changed_redirect_target_rotates_authority_even_with_the_same_etag() {
    let mut catalog = Catalog::new();
    let identity = catalog
        .upsert(PostId::new("redirect-generation"), metadata())
        .transfer(SOURCE)
        .unwrap();
    assert!(catalog
        .learn_response_observation_for(&identity, response("https://a.example/video.mp4", 1),));
    let first = catalog.http_generation_for(&identity).unwrap();
    assert!(catalog
        .learn_response_observation_for(&identity, response("https://b.example/video.mp4", 2),));

    assert_ne!(catalog.http_generation_for(&identity), Some(first));
}

fn response(final_url: &str, order: u64) -> HttpObservation {
    HttpObservation::new(
        LearnedFacts::default(),
        None,
        EvidenceTime::ordered(200, order),
        EvidenceValidator::strong_etag("\"stable\""),
    )
    .with_final_url(final_url)
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
