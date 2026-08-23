use ghostr_engine::catalog::{Catalog, HttpObservation, LearnedFacts};
use ghostr_engine::evidence::{EvidenceTime, EvidenceValidator};
use ghostr_engine::representation::HttpGenerationAuthority;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

const SOURCE: &str = "https://media.example/video.mp4";
const REDIRECTED: &str = "https://cdn.example/video.mp4";

#[test]
fn validatorless_head_redirect_has_a_stable_unknown_stamp() {
    let (mut catalog, identity) = fixture();
    assert!(catalog.learn_response_observation_for(
        &identity,
        observation(SOURCE, EvidenceValidator::strong_etag("\"v1\""), 1),
    ));

    let first = catalog
        .learn_head_observation_with_stamp_for(&identity, observation(REDIRECTED, None, 2))
        .expect("accepted redirect HEAD");
    assert_eq!(first.key().final_url(), REDIRECTED);
    assert!(matches!(
        first.authority(),
        HttpGenerationAuthority::Unknown(_)
    ));
    let repeated = catalog
        .learn_head_observation_with_stamp_for(&identity, observation(REDIRECTED, None, 3))
        .expect("repeated redirect HEAD");

    assert_eq!(repeated, first);
}

#[test]
fn stale_head_never_inherits_a_newer_response_stamp() {
    let (mut catalog, identity) = fixture();
    assert!(catalog.learn_response_observation_for(
        &identity,
        observation(REDIRECTED, EvidenceValidator::strong_etag("\"v2\""), 2),
    ));

    assert!(catalog
        .learn_head_observation_with_stamp_for(
            &identity,
            observation(SOURCE, EvidenceValidator::strong_etag("\"v1\""), 1),
        )
        .is_none());
}

fn fixture() -> (Catalog, ghostr_engine::representation::TransferIdentity) {
    let post = PostId::new("post");
    let mut catalog = Catalog::new();
    let identity = catalog.upsert(post, metadata()).transfer(SOURCE).unwrap();
    (catalog, identity)
}

fn observation(
    final_url: &str,
    validator: Option<EvidenceValidator>,
    order: u64,
) -> HttpObservation {
    HttpObservation::new(
        LearnedFacts::default(),
        Some("video/mp4".into()),
        EvidenceTime::ordered(100, order),
        validator,
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
