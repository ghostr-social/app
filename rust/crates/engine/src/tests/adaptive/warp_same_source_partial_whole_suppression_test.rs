use super::streamable_partial_fixture::{partial_state, TOTAL};
use crate::adaptive::{
    AdaptivePlayabilityPolicy, MediaLayout, PlannerCapability, PlannerCommand, PlannerContext,
    RetrievalRequest, WarpPlanner, WarpPlannerInput, REQUEST_SLICE_BYTES,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::healthy_origin;
use crate::ByteRange;

/// The rescue source of a next post whose primary origin was retired.
const RESCUE_SOURCE: &str = "https://rescue.example/next-rescue.mp4";
/// Startup prefix plus the first-frame tail already stored on the device.
const STORED_PREFIX_END: u64 = 70_258;
const DEVICE_ORIGIN_BPS: u64 = 1_500_000;
const DEVICE_ORIGIN_LATENCY_MS: u64 = 40;

#[test]
fn single_slice_completion_on_the_same_source_suppresses_the_whole_fetch() {
    let decision = plan(&rescue_state());
    let rescue = transfers(&decision);

    assert!(
        !rescue
            .iter()
            .any(|request| matches!(request, RetrievalRequest::FetchWhole { .. })),
        "{rescue:#?}"
    );
    assert!(
        rescue.iter().any(|request| matches!(request,
            RetrievalRequest::FetchRange { bytes, .. }
                if *bytes == ByteRange::new(STORED_PREFIX_END, TOTAL))),
        "{rescue:#?}"
    );
}

#[test]
fn multi_slice_completion_on_the_same_source_keeps_the_whole_crossover() {
    let mut state = rescue_state();
    state.request_slice_bytes = 65_536;
    let decision = plan(&state);
    let rescue = transfers(&decision);

    assert!(
        rescue
            .iter()
            .any(|request| matches!(request, RetrievalRequest::FetchWhole { .. })),
        "{rescue:#?}"
    );
}

fn rescue_state() -> crate::adaptive::PlayabilitySnapshot {
    let mut state = partial_state(TOTAL);
    state.request_slice_bytes = REQUEST_SLICE_BYTES;
    state.playback.buffer_ahead_ms = 0;
    let candidate = &mut state.candidates[1];
    candidate.layout = MediaLayout::Unknown;
    candidate.present = vec![ByteRange::new(0, STORED_PREFIX_END)];
    candidate.preferred_source = Some(RESCUE_SOURCE.into());
    candidate.origins = vec![healthy_origin(
        RESCUE_SOURCE,
        DEVICE_ORIGIN_BPS,
        DEVICE_ORIGIN_LATENCY_MS,
    )];
    state
}

fn plan(state: &crate::adaptive::PlayabilitySnapshot) -> crate::adaptive::WarpPlanningDecision {
    let base = AdaptivePlayabilityPolicy.plan(state);
    let context = PlannerContext::explicitly_unavailable(state).with_capability(
        &state.candidates[1].post,
        PlannerCapability::reported(true, None, 1),
    );
    WarpPlanner::default().plan(WarpPlannerInput::new(
        state,
        &base,
        &OriginModel::default(),
        &context,
    ))
}

fn transfers(decision: &crate::adaptive::WarpPlanningDecision) -> Vec<&RetrievalRequest> {
    decision
        .generated
        .actions
        .iter()
        .filter_map(|action| match &action.command {
            PlannerCommand::Transfer(allocation) if allocation.source == RESCUE_SOURCE => {
                Some(&allocation.request)
            }
            _ => None,
        })
        .collect()
}
