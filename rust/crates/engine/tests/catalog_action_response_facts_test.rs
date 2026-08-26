use crate::catalog::{Catalog, HttpObservation, LearnedFacts};
use crate::evidence::{EvidenceTime, EvidenceValidator};
use crate::{DeliveryKind, PostId, VideoMeta};

const URL: &str = "https://media.example/video.mp4";

#[test]
fn validatorless_response_facts_do_not_claim_or_revoke_http_authority() {
    let post = PostId::new("action-response-facts");
    let mut catalog = Catalog::new();
    let identity = catalog
        .upsert(post.clone(), metadata())
        .transfer(URL)
        .expect("valid test fixture");
    assert!(catalog.learn_response_observation_for(&identity, trusted_response()));
    let authority = catalog
        .http_generation_for(&identity)
        .expect("valid test fixture");

    assert!(catalog.learn_action_response_observation_for(
        &identity,
        HttpObservation::new(
            LearnedFacts {
                content_length: Some(32),
                accept_ranges: Some(false),
                host: Some("redirect.example".into()),
            },
            Some("video/mp4".into()),
            EvidenceTime::ordered(100, 2),
            None,
        )
        .with_final_url("https://redirect.example/video.mp4"),
    ));

    assert_eq!(catalog.http_generation_for(&identity), Some(authority));
    let entry = catalog.lookup(&post).expect("valid test fixture");
    assert_eq!(entry.authoritative_total_for(URL), Some(32));
    assert_eq!(entry.observed_range_support_for(URL), Some(false));
}

fn trusted_response() -> HttpObservation {
    HttpObservation::new(
        LearnedFacts {
            content_length: Some(16),
            accept_ranges: Some(true),
            host: Some("media.example".into()),
        },
        None,
        EvidenceTime::ordered(100, 1),
        EvidenceValidator::strong_etag("\"v1\""),
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
