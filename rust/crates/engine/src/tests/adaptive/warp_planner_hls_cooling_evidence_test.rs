use crate::adaptive::axiom_test_support::WarpActionGenerator;
use crate::adaptive::{
    AdaptivePlayabilityPolicy, FeedOffset, HlsBootstrapStage, HlsBootstrapState,
    HlsCandidateSnapshot, HlsObjectCursor, HlsTransport, PlannerContext, PlannerRetryAvailability,
    PlannerRetryEvidence, ViewProbability, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::{ActionId, PostId};

#[test]
fn cooling_hls_candidate_is_suppressed_and_recorded_for_replay() {
    let mut state = snapshot(0, 80_000_000, 0, 0);
    let post = PostId::new("hls");
    state.hls_candidates.push(candidate(post.clone()));
    let availability = PlannerRetryAvailability::Cooling {
        eligible_at_ms: 42_000,
    };
    let context =
        PlannerContext::explicitly_unavailable(&state).with_retry_availability(&post, availability);

    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &state,
        &AdaptivePlayabilityPolicy.plan(&state),
        &OriginModel::default(),
        &context,
    ));

    assert!(decision.generated.actions.is_empty());
    assert_eq!(
        decision.retry_availability,
        vec![PlannerRetryEvidence::new(post, availability)]
    );
}

#[test]
fn cooling_does_not_block_an_already_open_hls_response() {
    let mut state = snapshot(0, 80_000_000, 0, 0);
    let post = PostId::new("hls");
    let mut live = candidate(post.clone());
    live.cursor = HlsObjectCursor::new(
        7,
        256 * 1024,
        Some(300 * 1024),
        HlsTransport::ContinueLive {
            response: ActionId::new(9),
        },
    );
    state.hls_candidates.push(live);
    let context = PlannerContext::explicitly_unavailable(&state).with_retry_availability(
        &post,
        PlannerRetryAvailability::Cooling {
            eligible_at_ms: 42_000,
        },
    );

    let generated = WarpActionGenerator::generate(
        &state,
        &AdaptivePlayabilityPolicy.plan(&state),
        &OriginModel::default(),
        &context,
    );

    assert_eq!(generated.actions.len(), 1);
    assert_eq!(generated.actions[0].node.resources.requests, 0);
}

fn candidate(post: PostId) -> HlsCandidateSnapshot {
    HlsCandidateSnapshot {
        post,
        feed_offset: FeedOffset::new(0),
        view_probability: ViewProbability::new(1.0).expect("valid test fixture"),
        startup_value_ms: 2_000,
        cursor: Default::default(),
        player_preparation: Default::default(),
        state: HlsBootstrapState::Pending {
            stage: HlsBootstrapStage::RootManifest,
            source: "https://hls.example/root.m3u8".to_owned(),
        },
    }
}
