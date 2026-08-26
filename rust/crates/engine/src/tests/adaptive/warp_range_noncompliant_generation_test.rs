use super::warp_range_noncompliant_unknown_size_generation_test::range_blind_candidate_with_size;
use crate::adaptive::{
    ActionKind, AdaptivePlayabilityPolicy, HeadProbeHistory, MediaLayout, PlannerContext,
    RetrievalRequest, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn complete_file_layout_excludes_ranges_and_selects_the_whole_body() {
    let candidate = range_blind_candidate_with_size(Some(3_750_000));
    assert_eq!(candidate.layout, MediaLayout::RequiresCompleteFile);
    let total = candidate.total_bytes.expect("known total");
    let mut input = snapshot(1, 20_000_000, 0, 0);
    let post = candidate.post.clone();
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
    let selected = decision.selected.expect("whole fetch remains admissible");
    assert_eq!(
        selected.node.kind,
        ActionKind::FetchWhole {
            maximum_bytes: total
        }
    );
    assert_eq!(
        selected.node.forecast.ready_playback_ms,
        input.candidates[0].duration_ms
    );
    assert!(selected.node.value.reserve_gain_micros > 0);
    assert!(selected.node.value.cache_gain_micros > 0);
    assert!(matches!(
        selected.command,
        crate::adaptive::PlannerCommand::Transfer(work)
            if work.expected_playable_gain_ms == input.candidates[0].duration_ms
                && work.utility.additional_playable_ms == input.candidates[0].duration_ms
                && matches!(work.request, RetrievalRequest::FetchWhole { contract, .. }
                if contract.maximum_bytes() == total)
    ));
}
