use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::probe::pool::{MetadataProbePool, ProbeClaimQuery};
use ghostr_engine::catalog::{Catalog, HttpObservation, LearnedFacts};
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

const SOURCE: &str = "https://media.example/video.mp4";

#[test]
fn completed_head_history_rearms_when_a_validator_invalidates_size() {
    let post = PostId::new("post");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(post.clone(), metadata());
    let identity = binding.transfer(SOURCE).expect("valid test fixture");
    assert!(catalog.learn_head_observation_for(
        &identity,
        observation(Some(16), Some(true), "\"generation-1\"", 1)
    ));
    let mut probes = MetadataProbePool::new(1);
    probes.learned(&identity, catalog.http_generation_for(&identity));
    assert!(catalog.learn_response_observation_for(
        &identity,
        observation(None, None, "\"generation-2\"", 2)
    ));
    assert_eq!(
        catalog
            .lookup(&post)
            .expect("valid test fixture")
            .conservative_size_for(SOURCE, 2)
            .exact,
        Some(16)
    );

    let retry = RetryBook::new(RetryPolicy::default());
    let query = ProbeClaimQuery {
        catalog: &catalog,
        retry: &retry,
        post: &post,
        source: SOURCE,
        observed_at_ms: 2,
    };
    assert!(probes.claim_selected(query).is_ok());
}

fn observation(
    size: Option<u64>,
    ranges: Option<bool>,
    etag: &str,
    at_ms: u64,
) -> HttpObservation {
    HttpObservation::new(
        LearnedFacts {
            content_length: size,
            accept_ranges: ranges,
            host: None,
        },
        Some("video/mp4".to_owned()),
        at_ms,
        EvidenceValidator::strong_etag(etag),
    )
}

fn metadata() -> VideoMeta {
    VideoMeta {
        urls: vec![SOURCE.into()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(16),
        duration_ms: None,
    }
}
