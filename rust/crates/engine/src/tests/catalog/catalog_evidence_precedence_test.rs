use crate::catalog::{Catalog, LearnedFacts};
use crate::tests::support::progressive_meta;
use crate::PostId;

#[test]
fn observed_response_dominates_later_advisory_head_fields() {
    let post = PostId::new("post");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(post.clone(), progressive_meta(None, None));
    let source = "https://host.example/video.mp4";
    let identity = binding.transfer(source).unwrap();
    assert!(catalog.learn_response_for(
        &identity,
        LearnedFacts {
            content_length: Some(16),
            accept_ranges: Some(true),
            host: None,
        },
    ));

    assert!(catalog.learn_head_for(
        &identity,
        LearnedFacts {
            content_length: Some(8),
            accept_ranges: Some(false),
            host: None,
        },
    ));

    let entry = catalog.lookup(&post).unwrap();
    assert_eq!(entry.planning_total_for(source), Some(16));
    assert_eq!(entry.authoritative_total_for(source), Some(16));
    assert_eq!(entry.observed_range_support_for(source), Some(true));
}
