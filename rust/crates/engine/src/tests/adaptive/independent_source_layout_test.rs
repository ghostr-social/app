use crate::adaptive::{
    candidate_snapshot, CandidateEvidence, FeedOffset, MediaLayout, ViewProbability,
};
use crate::catalog::{Catalog, LearnedFacts};
use crate::tests::adaptive_support::healthy_origin;
use crate::{DeliveryKind, EngineParams, PostId, VideoMeta};
use std::collections::HashSet;

#[test]
fn incompatible_range_source_replans_as_one_independent_object() {
    let post = PostId::new("post");
    let primary = "https://a.example/video";
    let mirror = "https://b.example/video";
    let mut catalog = Catalog::new();
    catalog.upsert(post.clone(), meta(primary, mirror));
    let mirror_id = catalog.transfer_identity(&post, mirror).unwrap();
    catalog.learn_response_for(
        &mirror_id,
        LearnedFacts {
            content_length: Some(16),
            accept_ranges: Some(true),
            host: None,
        },
    );
    let candidate = candidate_snapshot(
        &catalog,
        &EngineParams::default(),
        CandidateEvidence {
            post,
            feed_offset: FeedOffset::new(0),
            view_probability: ViewProbability::new(1.0).unwrap(),
            present: vec![crate::ByteRange::new(0, 4)],
            stored_total: Some(16),
            continuation_source: Some(primary.to_owned()),
            independent_object_sources: HashSet::from([mirror.to_owned()]),
            recently_evicted: Vec::new(),
            in_flight: Vec::new(),
            origins: vec![healthy_origin(mirror, 1_000_000, 20)],
        },
    )
    .unwrap();

    assert_eq!(candidate.preferred_source.as_deref(), Some(mirror));
    assert_eq!(candidate.layout, MediaLayout::RequiresCompleteFile);
    assert!(candidate.present.is_empty());
}

fn meta(primary: &str, mirror: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![primary.to_owned(), mirror.to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(16),
        duration_ms: Some(1_000),
    }
}
