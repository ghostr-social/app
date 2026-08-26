use crate::adaptive::axiom_test_support::WarpActionGenerator;
use crate::adaptive::{ActionKind, AdaptivePlayabilityPolicy, PlannerContext};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::ByteRange;

#[test]
fn present_timeline_probe_never_generates_a_duplicate_tail() {
    let mut input = snapshot(1, 8_000_000, 1_000, 20);
    let candidate = &mut input.candidates[0];
    let probe = candidate.playable_ranges.last().expect("tail probe").bytes;
    let total = candidate.total_bytes.expect("known test size");
    candidate.timeline_probe = Some(crate::adaptive::PlayableRange {
        bytes: probe,
        playable_ms: 4_000,
    });
    candidate.present = vec![ByteRange::new(0, total)];
    candidate.finalized = true;
    let base = AdaptivePlayabilityPolicy.plan(&input);
    assert!(base.allocations.is_empty(), "fully present baseline");

    let generated = WarpActionGenerator::generate(
        &input,
        &base,
        &OriginModel::default(),
        &PlannerContext::explicitly_unavailable(&input),
    );

    assert!(generated
        .actions
        .iter()
        .all(|action| action.node.kind != ActionKind::Tail(probe)));
}
