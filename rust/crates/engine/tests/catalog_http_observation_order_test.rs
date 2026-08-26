use crate::catalog::{Catalog, HttpObservation, LearnedFacts};
use crate::evidence::{EvidenceField, EvidenceTime, EvidenceValidator, EvidenceValue};
use crate::{DeliveryKind, PostId, VideoMeta};

const URL: &str = "https://media.example/video.mp4";

#[test]
fn cross_authority_observations_cannot_lower_the_validator_clock() {
    let post = PostId::new("ordered-http");
    let mut catalog = Catalog::new();
    let identity = catalog
        .upsert(post.clone(), metadata())
        .transfer(URL)
        .expect("valid test fixture");
    assert!(catalog.learn_response_observation_for(&identity, observation(200, "v2", Some(false))));
    assert!(catalog.learn_head_observation_for(&identity, observation(100, "v2", Some(true))));

    assert!(!catalog.learn_head_observation_for(&identity, observation(150, "v1", Some(true))));
    let assessment = catalog
        .lookup(&post)
        .expect("valid test fixture")
        .evidence_assessment_for(URL, 201);
    assert_eq!(
        assessment.value(EvidenceField::RangeSupport),
        Some(&EvidenceValue::RangeSupport(false))
    );
}

#[test]
fn monotonic_order_wins_when_the_wall_clock_moves_backward() {
    let post = PostId::new("wall-clock-rollback");
    let mut catalog = Catalog::new();
    let identity = catalog
        .upsert(post.clone(), metadata())
        .transfer(URL)
        .expect("valid test fixture");
    assert!(catalog
        .learn_response_observation_for(&identity, ordered_observation(200, 1, "v2", Some(false))));
    assert!(catalog
        .learn_response_observation_for(&identity, ordered_observation(100, 2, "v2", Some(true))));

    let assessment = catalog
        .lookup(&post)
        .expect("valid test fixture")
        .evidence_assessment_for(URL, 201);
    assert_eq!(
        assessment.value(EvidenceField::RangeSupport),
        Some(&EvidenceValue::RangeSupport(true))
    );
}

fn observation(order: u64, etag: &str, ranges: Option<bool>) -> HttpObservation {
    ordered_observation(100, order, etag, ranges)
}

fn ordered_observation(
    wall_ms: u64,
    order: u64,
    etag: &str,
    ranges: Option<bool>,
) -> HttpObservation {
    HttpObservation::new(
        LearnedFacts {
            accept_ranges: ranges,
            ..LearnedFacts::default()
        },
        None,
        EvidenceTime::ordered(wall_ms, order),
        EvidenceValidator::strong_etag(format!("\"{etag}\"")),
    )
}

fn metadata() -> VideoMeta {
    VideoMeta {
        urls: vec![URL.to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: None,
        duration_ms: None,
    }
}
