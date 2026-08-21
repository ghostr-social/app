use crate::adaptive::{
    ActionKind, ActivePlannerContext, Allocation, AllocationPlan, AllocationReason,
    CandidateUtility, GeneratedAction, HedgeInput, IdentityProof, InFlightAction, PlannerCommand,
    PlannerContext, PreemptionAuthority, PromotionGrant, RetrievalRequest, WarpActionGenerator,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::{ActionId, ByteRange};

const SOURCE: &str = "https://origin.example/media";
const MIRROR: &str = "https://mirror.example/media";
const RESERVED_BYTES: u64 = 800_000;
#[test]
fn transfer_charges_only_the_immediate_range_before_promotion() {
    let input = snapshot(1, 8_000_000, 1_000, 20);
    let allocation = allocation(input.candidates[0].post.clone());
    let base = AllocationPlan {
        allocations: vec![allocation],
        ..AllocationPlan::default()
    };
    let context = PlannerContext::explicitly_unavailable(&input);
    let generated = WarpActionGenerator::generate(&input, &base, &OriginModel::default(), &context);
    let action = generated.actions.iter().find(|action| {
        matches!(&action.command, PlannerCommand::Transfer(work) if work.request == request())
    });
    assert_immediate(action.expect("promotable transfer"));
}
#[test]
fn hedge_charges_only_its_immediate_duplicate_range() {
    let mut input = snapshot(1, 8_000_000, 1_000, 20);
    let active_id = ActionId::new(17);
    let range = ByteRange::new(0, 64_000);
    let mut active = InFlightAction::range(active_id, range, SOURCE, 20_000, true);
    active.request = request();
    input.candidates[0].in_flight.push(active);
    let hedge = HedgeInput::new(active_id, ActionKind::FetchRange(range))
        .with_timing(1_000, 900)
        .with_value(5_000, 1_000)
        .with_network_envelope(RESERVED_BYTES);
    let post = input.candidates[0].post.clone();
    let active = ActivePlannerContext::new(active_id, post).with_hedge(
        hedge,
        IdentityProof::VerifiedHash([3; 32]),
        MIRROR,
    );
    let context = PlannerContext::explicitly_unavailable(&input).with_active(active);
    let generated = WarpActionGenerator::generate(
        &input,
        &AllocationPlan::default(),
        &OriginModel::default(),
        &context,
    );
    let action = generated
        .actions
        .iter()
        .find(|action| matches!(&action.command, PlannerCommand::Hedge { transfer, .. } if transfer.request == request()));
    assert_immediate(action.expect("promotable hedge"));
}
fn assert_immediate(action: &GeneratedAction) {
    let request = match &action.command {
        PlannerCommand::Transfer(work) | PlannerCommand::Hedge { transfer: work, .. } => {
            work.request
        }
        _ => panic!("expected request command"),
    };
    let immediate = request.immediate_network_bytes();
    assert_ne!(request.reserved_network_bytes(), immediate);
    assert_eq!(action.node.resources.network_bytes, immediate);
    assert_eq!(action.node.resources.storage_bytes, immediate);
    assert_eq!(action.node.resources.cpu_ms, 0);
    assert_eq!(action.node.resources.requests, 1);
}

fn request() -> RetrievalRequest {
    RetrievalRequest::FetchRange {
        bytes: ByteRange::new(0, 64_000),
        promotion: Some(PromotionGrant {
            maximum_bytes: RESERVED_BYTES,
            valid_until_ms: 20_000,
        }),
    }
}

fn allocation(post: crate::PostId) -> Allocation {
    Allocation {
        post,
        request: request(),
        source: SOURCE.to_owned(),
        expected_playable_gain_ms: 1_000,
        utility: CandidateUtility {
            view_probability: 1.0,
            additional_playable_ms: 1_000,
            expected_delivery_ms: 10,
            score: 1.0,
        },
        authority: PreemptionAuthority::PlaybackCritical,
        commitment_until_ms: 20_000,
        reason: AllocationReason::MediaBootstrap,
    }
}
