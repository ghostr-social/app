use crate::adaptive::axiom_test_support::WarpActionGenerator;
use crate::adaptive::{
    ActionKind, AdaptivePlayabilityPolicy, HeadProbeHistory, MediaLayout, PlannerContext,
    PlayableRange,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::tests::support::set_reliable_total_bytes;
use crate::ByteRange;

const FIRST_PROBE_END: u64 = 65_536;

#[test]
fn unknown_layout_continues_discovery_after_the_first_probe() {
    let mut input = snapshot(1, 20_000_000, 0, 0);
    let candidate = &mut input.candidates[0];
    candidate.layout = MediaLayout::Unknown;
    candidate.present = vec![ByteRange::new(0, FIRST_PROBE_END)];
    candidate.playable_ranges = vec![PlayableRange {
        bytes: ByteRange::new(FIRST_PROBE_END, 285_652),
        playable_ms: 1,
    }];
    set_reliable_total_bytes(candidate, 285_652, input.observed_at_ms);
    let post = candidate.post.clone();
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let context = PlannerContext::explicitly_unavailable(&input)
        .with_head_probe_history(&post, HeadProbeHistory::Completed);

    let generated = WarpActionGenerator::generate(&input, &base, &OriginModel::default(), &context);

    assert!(generated.actions.iter().any(|action| {
        action.node.kind == ActionKind::Prefix(ByteRange::new(FIRST_PROBE_END, 2 * FIRST_PROBE_END))
    }));
}
