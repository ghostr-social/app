use crate::adaptive::{
    candidate_snapshot, AdaptivePlayabilityPolicy, CandidateEvidence, ControlMode, FeedOffset,
    NextReserveEvidence, ViewProbability,
};
use crate::catalog::{Catalog, LearnedFacts};
use crate::media_timeline::{parse_mp4_segments, MediaSegment};
use crate::tests::adaptive_support::{healthy_origin, snapshot};
use crate::tests::media_timeline_support::{classic_mdat_prefix, classic_moov, valid_ftyp};
use crate::tests::support::progressive_meta;
use crate::{ByteRange, EngineParams, PostId};

#[test]
fn adjacent_startup_requires_initialization_and_first_media() {
    let ftyp = valid_ftyp();
    let moov = classic_moov(&[100, 500], &[100, 100]);
    let mut input = snapshot(2, 20_000_000, 20_000, 2);
    input.candidates[0].present = input.candidates[0]
        .playable_ranges
        .iter()
        .take(5)
        .map(|range| range.bytes)
        .collect();

    input.candidates[1] = candidate(&ftyp, &moov, metadata(&moov));
    let metadata_plan = AdaptivePlayabilityPolicy.plan(&input);
    assert!(!matches!(
        metadata_plan.next_reserve,
        NextReserveEvidence::Structural { .. }
    ));

    let movie = ByteRange::new(10_000, 10_000 + moov.len() as u64);
    input.candidates[1] = candidate(
        &ftyp,
        &moov,
        vec![ByteRange::new(0, 24), ByteRange::new(100, 200)],
    );
    let missing_movie = AdaptivePlayabilityPolicy.plan(&input);
    assert!(missing_movie.allocations.iter().any(|work| {
        work.post == PostId::new("p1") && overlaps(work.request.requested_bytes(), movie)
    }));

    let mut complete = metadata(&moov);
    complete.push(ByteRange::new(100, 200));
    input.candidates[1] = candidate(&ftyp, &moov, complete);
    let plan = AdaptivePlayabilityPolicy.plan(&input);
    let NextReserveEvidence::Structural { post, startup } = plan.next_reserve else {
        panic!("exact sparse startup closure was not structural");
    };
    assert_eq!(post, PostId::new("p1"));
    assert_eq!(plan.ready_reserve.ready, 0);
    assert_eq!(plan.ready_reserve.structural, 1);
    assert_eq!(plan.ready_reserve.protected, 1);
    assert_eq!(plan.mode, ControlMode::Safety);
    assert_eq!(
        startup.ranges(),
        &[
            ByteRange::new(0, 24),
            ByteRange::new(100, 200),
            ByteRange::new(10_000, 10_000 + moov.len() as u64)
        ]
    );
}

fn candidate(
    ftyp: &[u8],
    moov: &[u8],
    present: Vec<ByteRange>,
) -> crate::adaptive::CandidateSnapshot {
    let post = PostId::new("p1");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(post.clone(), progressive_meta(Some(20_000), Some(2_000)));
    catalog.learn(
        &post,
        LearnedFacts {
            accept_ranges: Some(true),
            ..Default::default()
        },
    );
    let prefix = classic_mdat_prefix(ftyp, 10_000, 24);
    let timeline = parse_mp4_segments(&[
        MediaSegment::new(0, &prefix),
        MediaSegment::new(10_000, moov),
    ])
    .expect("valid test fixture");
    assert!(catalog.learn_timeline_for(&binding, timeline));
    candidate_snapshot(
        &catalog,
        &EngineParams::default(),
        CandidateEvidence {
            post,
            feed_offset: FeedOffset::new(1),
            view_probability: ViewProbability::new(0.8).expect("valid test fixture"),
            present,
            stored_total: Some(20_000),
            continuation_source: None,
            independent_object_sources: Default::default(),
            recently_evicted: Vec::new(),
            in_flight: Vec::new(),
            origins: vec![healthy_origin(
                "https://host.example/video.mp4",
                20_000_000,
                20,
            )],
        },
    )
    .expect("valid test fixture")
}

fn metadata(moov: &[u8]) -> Vec<ByteRange> {
    vec![
        ByteRange::new(0, 24),
        ByteRange::new(10_000, 10_000 + moov.len() as u64),
    ]
}

fn overlaps(left: ByteRange, right: ByteRange) -> bool {
    left.start < right.end && right.start < left.end
}
