use crate::adaptive::{
    AdaptivePlayabilityPolicy, GeneratedActions, WarpPlanner, WarpPlannerInput,
    WarpPlanningDecision,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::{healthy_origin, snapshot};
use crate::tests::support::set_reliable_total_bytes;
use crate::{ActionId, ByteRange};

type Snapshot = crate::adaptive::PlayabilitySnapshot;
const ORIGIN: &str = "https://origin.example/media";
const MIRROR: &str = "https://mirror.example/media";

#[path = "warp_planner_promotion_support/active.rs"]
mod active_support;
#[path = "warp_planner_promotion_support/context.rs"]
mod context_support;

pub(in crate::tests::adaptive::warp_tests) fn generated_actions(
    observed_body_bytes: Option<u64>,
) -> GeneratedActions {
    planned(observed_body_bytes).1.generated
}

pub(super) fn planned(observed_body_bytes: Option<u64>) -> (Snapshot, WarpPlanningDecision) {
    planned_with_model(observed_body_bytes, &OriginModel::default())
}

pub(super) fn planned_with_model(
    observed_body_bytes: Option<u64>,
    origins: &OriginModel,
) -> (Snapshot, WarpPlanningDecision) {
    planned_with_reservation(observed_body_bytes, origins, None)
}

fn planned_with_reservation(
    observed_body_bytes: Option<u64>,
    origins: &OriginModel,
    reserved_storage_bytes: Option<u64>,
) -> (Snapshot, WarpPlanningDecision) {
    let mut input = snapshot(1, 8_000_000, 1_000, 20);
    let observed_at_ms = input.observed_at_ms;
    let post = configure_candidate(&mut input, observed_at_ms);
    let active_id = ActionId::new(17);
    input.candidates[0]
        .in_flight
        .push(active_support::active_range(
            active_id,
            observed_body_bytes,
            reserved_storage_bytes,
            ORIGIN,
        ));
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let context = context_support::context(&input, active_id, &post, MIRROR);
    let planner_input = WarpPlannerInput::new(&input, &base, origins, &context);
    let decision = WarpPlanner::default().plan(planner_input);
    (input, decision)
}

fn configure_candidate(input: &mut Snapshot, observed_at_ms: u64) -> crate::PostId {
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
}
