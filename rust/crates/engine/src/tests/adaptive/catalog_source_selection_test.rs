use crate::adaptive::{
    candidate_snapshot, CandidateEvidence, FeedOffset, MediaLayout, ViewProbability,
};
use crate::catalog::{Catalog, LearnedFacts};
use crate::tests::adaptive_support::healthy_origin;
use crate::{DeliveryKind, EngineParams, PostId, VideoMeta};

#[test]
fn candidate_geometry_uses_the_selected_origins_observed_semantics() {
    let post = PostId::new("mirrored");
    let primary = "https://primary/video.mp4";
    let mirror = "https://mirror/video.mp4";
    let mut catalog = Catalog::new();
    catalog.upsert(post.clone(), meta(primary, mirror));
    learn(&mut catalog, &post, primary, 8, false);
    learn(&mut catalog, &post, mirror, 16, true);

    let candidate = candidate_snapshot(
        &catalog,
        &EngineParams::default(),
        CandidateEvidence {
            post,
            feed_offset: FeedOffset::new(0),
            view_probability: ViewProbability::new(1.0).expect("valid test fixture"),
            present: Vec::new(),
            stored_total: None,
            continuation_source: None,
            independent_object_sources: Default::default(),
            recently_evicted: Vec::new(),
            in_flight: Vec::new(),
            origins: vec![
                healthy_origin(primary, 1_000_000, 50),
                healthy_origin(mirror, 10_000_000, 50),
            ],
        },
    )
    .expect("valid test fixture");

    assert_eq!(candidate.total_bytes, Some(16));
    assert_eq!(candidate.layout, MediaLayout::Streamable);
}

#[test]
fn equally_unknown_sources_keep_the_publishers_order() {
    let post = PostId::new("ordered-mirrors");
    let primary = "https://primary/video.mp4";
    let blossom = "https://blossom/hash";
    let mut catalog = Catalog::new();
    catalog.upsert(post.clone(), meta(primary, blossom));

    let candidate = candidate_snapshot(
        &catalog,
        &EngineParams::default(),
        CandidateEvidence {
            post,
            feed_offset: FeedOffset::new(0),
            view_probability: ViewProbability::new(1.0).expect("valid test fixture"),
            present: Vec::new(),
            stored_total: None,
            continuation_source: None,
            independent_object_sources: Default::default(),
            recently_evicted: Vec::new(),
            in_flight: Vec::new(),
            origins: vec![
                healthy_origin(primary, 1_000_000, 50),
                healthy_origin(blossom, 1_000_000, 50),
            ],
        },
    )
    .expect("valid test fixture");

    assert_eq!(candidate.preferred_source.as_deref(), Some(primary));
}

fn learn(catalog: &mut Catalog, post: &PostId, source: &str, total: u64, ranged: bool) {
    let identity = catalog
        .transfer_identity(post, source)
        .expect("valid test fixture");
    assert!(catalog.learn_response_for(
        &identity,
        LearnedFacts {
            content_length: Some(total),
            accept_ranges: Some(ranged),
            host: None,
        },
    ));
}

fn meta(primary: &str, mirror: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![primary.to_owned(), mirror.to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: None,
        duration_ms: Some(1_000),
    }
}
