use crate::adaptive::{
    candidate_snapshot_at, ActionKind, AdaptivePlayabilityPolicy, CandidateEvidence, FeedOffset,
    HeadProbeHistory, MediaLayout, PlannerContext, RetrievalRequest, ViewProbability, WarpPlanner,
    WarpPlannerInput, BOOTSTRAP_DIRECT_FETCH_BYTES,
};
use crate::catalog::{Catalog, HttpObservation, LearnedFacts};
use crate::evidence::EvidenceValidator;
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::{healthy_origin, snapshot};
use crate::tests::support::progressive_meta;
use crate::{EngineParams, PostId};

const SOURCE: &str = "https://host.example/video.mp4";

#[test]
fn unknown_size_complete_file_uses_a_capped_whole_fetch() {
    let candidate = range_blind_candidate();
    assert_eq!(candidate.layout, MediaLayout::RequiresCompleteFile);
    assert_eq!(candidate.total_bytes, None);
    let post = candidate.post.clone();
    let mut input = snapshot(1, 20_000_000, 0, 0);
    input.candidates = vec![candidate];
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let context = PlannerContext::explicitly_unavailable(&input)
        .with_head_probe_history(&post, HeadProbeHistory::Completed);

    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &input,
        &base,
        &OriginModel::default(),
        &context,
    ));

    assert!(decision.generated.actions.iter().all(|action| !matches!(
        action.node.kind,
        ActionKind::Prefix(_)
            | ActionKind::Tail(_)
            | ActionKind::FetchRange(_)
            | ActionKind::CacheUpgrade(_)
    )));
    let selected = decision.selected.expect("bounded whole fetch");
    assert_eq!(
        selected.node.kind,
        ActionKind::FetchWhole {
            maximum_bytes: BOOTSTRAP_DIRECT_FETCH_BYTES
        }
    );
    assert_eq!(selected.node.forecast.ready_playback_ms, 0);
    assert!(matches!(
        selected.command,
        crate::adaptive::PlannerCommand::Transfer(work)
            if work.expected_playable_gain_ms == 0
                && work.utility.additional_playable_ms == 0
                && matches!(work.request, RetrievalRequest::FetchWhole { contract, .. }
                    if contract.maximum_bytes() == BOOTSTRAP_DIRECT_FETCH_BYTES)
    ));
}

pub(super) fn range_blind_candidate() -> crate::adaptive::CandidateSnapshot {
    range_blind_candidate_with_size(None)
}

pub(super) fn range_blind_candidate_with_size(
    size: Option<u64>,
) -> crate::adaptive::CandidateSnapshot {
    let post = PostId::new("p0");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(post.clone(), progressive_meta(size, Some(60_000)));
    let identity = binding.transfer(SOURCE).expect("source identity");
    let observation = HttpObservation::new(
        LearnedFacts {
            accept_ranges: Some(false),
            ..LearnedFacts::default()
        },
        None,
        10_000,
        EvidenceValidator::strong_etag("\"generation-1\""),
    );
    assert!(catalog.learn_response_observation_for(&identity, observation));
    candidate_snapshot_at(
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
            origins: vec![healthy_origin(SOURCE, 20_000_000, 50)],
        },
        10_000,
    )
    .expect("range-blind candidate")
}
