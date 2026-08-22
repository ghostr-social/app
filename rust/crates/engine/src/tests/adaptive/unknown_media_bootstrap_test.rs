use crate::adaptive::{
    candidate_snapshot, AllocationReason, CandidateEvidence, FeedOffset, ViewProbability,
};
use crate::catalog::Catalog;
use crate::tests::adaptive_support::{healthy_origin, snapshot};
use crate::{DeliveryKind, EngineParams, PostId, VideoMeta};

#[test]
fn cold_current_and_next_receive_typed_bounded_bootstrap_allocations() {
    let mut catalog = Catalog::new();
    for post in ["p0", "p1"] {
        catalog.upsert(PostId::new(post), unknown_meta(post));
    }
    let params = EngineParams::default();
    let mut input = snapshot(0, 700_000, 0, 2);
    input.network.connection_capacity = 1;
    input.network.connection_ceiling = 1;
    input.candidates = ["p0", "p1"]
        .into_iter()
        .enumerate()
        .map(|(distance, post)| candidate(&catalog, &params, post, distance))
        .collect();

    let plan = crate::adaptive::AdaptivePlayabilityPolicy.plan(&input);

    assert_eq!(plan.allocations[0].post, PostId::new("p0"));
    assert_eq!(plan.allocations[0].reason, AllocationReason::MediaBootstrap);
    let next = plan
        .allocations
        .iter()
        .find(|work| work.post == PostId::new("p1"))
        .expect("immediate-next bootstrap");
    assert_eq!(next.reason, AllocationReason::MediaBootstrap);
    assert!(next.request.requested_bytes().len() <= input.request_slice_bytes);
}

fn candidate(
    catalog: &Catalog,
    params: &EngineParams,
    post: &str,
    distance: usize,
) -> crate::adaptive::CandidateSnapshot {
    candidate_snapshot(
        catalog,
        params,
        CandidateEvidence {
            post: PostId::new(post),
            feed_offset: FeedOffset::new(distance as i32),
            view_probability: ViewProbability::new(1.0).unwrap(),
            present: Vec::new(),
            stored_total: None,
            continuation_source: None,
            independent_object_sources: Default::default(),
            recently_evicted: Vec::new(),
            in_flight: Vec::new(),
            origins: vec![healthy_origin(
                &format!("https://{post}.example/video.mp4"),
                700_000,
                450,
            )],
        },
    )
    .expect("unknown media remains a bootstrap candidate")
}

fn unknown_meta(post: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![format!("https://{post}.example/video.mp4")],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: None,
        duration_ms: None,
    }
}
