use crate::adaptive::{
    ActionKind, ActivePlannerContext, AdaptivePlayabilityPolicy, GeneratedActions, HedgeInput,
    IdentityProof, InFlightAction, PlannerCapability, PlannerContext, PromotionGrant,
    RetrievalRequest, TransformCapability, TransformKind, WarpActionGenerator,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::{healthy_origin, snapshot};
use crate::tests::support::set_reliable_total_bytes;
use crate::{ActionId, ByteRange};

const ORIGIN: &str = "https://origin.example/media";
const MIRROR: &str = "https://mirror.example/media";

#[test]
fn adaptive_dag_generates_every_paper_action_from_explicit_evidence() {
    let generated = generated_actions();
    let kinds: Vec<_> = generated
        .actions
        .iter()
        .map(|item| &item.node.kind)
        .collect();
    assert!(kinds.iter().any(|kind| matches!(kind, ActionKind::Head)));
    assert!(kinds
        .iter()
        .any(|kind| matches!(kind, ActionKind::Prefix(_))));
    assert!(kinds.iter().any(|kind| matches!(kind, ActionKind::Tail(_))));
    assert!(kinds
        .iter()
        .any(|kind| matches!(kind, ActionKind::FetchRange(_))));
    assert!(kinds
        .iter()
        .any(|kind| matches!(kind, ActionKind::FetchWhole { .. })));
    assert!(kinds
        .iter()
        .any(|kind| matches!(kind, ActionKind::Promote { .. })));
    assert!(kinds
        .iter()
        .any(|kind| matches!(kind, ActionKind::Transform(_))));
    assert!(kinds
        .iter()
        .any(|kind| matches!(kind, ActionKind::CacheUpgrade(_))));
    assert!(kinds
        .iter()
        .any(|kind| matches!(kind, ActionKind::Hedge { .. })));
    assert!(kinds
        .iter()
        .any(|kind| matches!(kind, ActionKind::Cancel(_))));
}

pub(super) fn generated_actions() -> GeneratedActions {
    let mut input = snapshot(1, 8_000_000, 1_000, 20);
    let observed_at_ms = input.observed_at_ms;
    let post = {
        let candidate = &mut input.candidates[0];
        candidate.layout = crate::adaptive::MediaLayout::Unknown;
        set_reliable_total_bytes(candidate, 800_000, observed_at_ms);
        candidate.timeline_probe = Some(crate::adaptive::PlayableRange {
            bytes: ByteRange::new(736_000, 800_000),
            playable_ms: 0,
        });
        candidate.present = vec![ByteRange::new(0, 32_000)];
        candidate
            .origins
            .push(healthy_origin(MIRROR, 7_000_000, 60));
        candidate.post.clone()
    };
    let active_id = ActionId::new(17);
    let mut active =
        InFlightAction::range(active_id, ByteRange::new(0, 64_000), ORIGIN, 20_000, true);
    active.request = RetrievalRequest::FetchRange {
        bytes: ByteRange::new(0, 64_000),
        promotion: Some(PromotionGrant {
            maximum_bytes: 800_000,
            valid_until_ms: 20_000,
        }),
    };
    input.candidates[0].in_flight.push(active);
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let hedge = HedgeInput::new(active_id, ActionKind::FetchRange(ByteRange::new(0, 64_000)))
        .with_timing(1_000, 900)
        .with_value(5_000, 1_000)
        .with_network_envelope(800_000);
    let active = ActivePlannerContext::new(active_id, post.clone())
        .with_continuation_advantage(-100_000)
        .with_hedge(hedge, IdentityProof::VerifiedHash([3; 32]), MIRROR);
    let context = PlannerContext::explicitly_unavailable(&input)
        .with_capability(
            post,
            PlannerCapability::reported(
                false,
                Some(TransformCapability::new(TransformKind::Remux, 300, 900_000)),
                4,
            ),
        )
        .with_active(active);

    WarpActionGenerator::generate(&input, &base, &OriginModel::default(), &context)
}
