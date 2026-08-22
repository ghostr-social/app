use crate::adaptive::{
    ActionKind, AdaptivePlayabilityPolicy, HeadProbeHistory, MediaLayout, PlannerContext,
    RetrievalRequest, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::ByteRange;

#[test]
fn complete_file_layout_excludes_ranges_and_selects_the_whole_body() {
    let mut input = snapshot(1, 20_000_000, 0, 0);
    let candidate = &mut input.candidates[0];
    let total = candidate.total_bytes.expect("known total");
    candidate.layout = MediaLayout::RequiresCompleteFile;
    candidate.evidence.size.reliable = false;
    candidate.present = vec![ByteRange::new(0, 32_000)];
    candidate.timeline_probe = Some(crate::adaptive::PlayableRange {
        bytes: ByteRange::new(total - 64_000, total),
        playable_ms: 0,
    });
    assert!(candidate.needs_bootstrap());
    let post = candidate.post.clone();
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let context = PlannerContext::explicitly_unavailable(&input)
        .with_head_probe_history(post, HeadProbeHistory::Completed);

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
    assert!(matches!(
        selected.command,
        crate::adaptive::PlannerCommand::Transfer(work)
            if matches!(work.request, RetrievalRequest::FetchWhole { contract, .. }
                if contract.maximum_bytes() == total)
    ));
}
