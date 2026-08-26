use crate::catalog::{Catalog, HttpObservation, LearnedFacts};
use crate::evidence::EvidenceValidator;
use crate::tests::support::progressive_meta;
use crate::PostId;

#[test]
fn observed_response_dominates_later_advisory_head_fields() {
    let post = PostId::new("post");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(post.clone(), progressive_meta(None, None));
    let source = "https://host.example/video.mp4";
    let identity = binding.transfer(source).expect("valid test fixture");
    assert!(catalog.learn_response_observation_for(&identity, observation(16, true, 1)));
    assert!(catalog.learn_head_observation_for(&identity, observation(8, false, 2)));

    let entry = catalog.lookup(&post).expect("valid test fixture");
    assert_eq!(entry.planning_total_for(source), Some(16));
    assert_eq!(entry.authoritative_total_for(source), Some(16));
    assert_eq!(entry.observed_range_support_for(source), Some(true));
}

fn observation(content_length: u64, accept_ranges: bool, observed_at_ms: u64) -> HttpObservation {
    HttpObservation::new(
        LearnedFacts {
            content_length: Some(content_length),
            accept_ranges: Some(accept_ranges),
            host: None,
        },
        None,
        observed_at_ms,
        EvidenceValidator::strong_etag("\"generation-1\""),
    )
}
