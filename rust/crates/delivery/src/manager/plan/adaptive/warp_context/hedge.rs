use super::ActiveContextInput;
use ghostr_engine::adaptive::{
    ActionKind, ActivePlannerContext, ControlMode, HedgeInput, OriginHealth, RetrievalRequest,
};

mod forecast;
mod identity;

pub(super) fn apply(
    context: ActivePlannerContext,
    evidence: &ActiveContextInput<'_>,
) -> ActivePlannerContext {
    let Some(candidate) = evidence
        .snapshot
        .candidates
        .iter()
        .find(|item| item.post == *evidence.active.post())
    else {
        return context;
    };
    let Some((input, proof, alternate)) = hedge(evidence, candidate) else {
        return context;
    };
    context.with_hedge(input, proof, alternate)
}

fn hedge(
    evidence: &ActiveContextInput<'_>,
    candidate: &ghostr_engine::adaptive::CandidateSnapshot,
) -> Option<(HedgeInput, ghostr_engine::adaptive::IdentityProof, String)> {
    let active = evidence.active;
    if active.cancelling() || active.hedged() || !primary_is_current(evidence) {
        return None;
    }
    let primary = origin(candidate, active.identity().source().as_str())?;
    let alternate = alternate(evidence, candidate)?;
    let proof = identity::proof(evidence, candidate, &alternate.source)?;
    let input = hedge_input(evidence, candidate, primary, alternate);
    Some((input, proof, alternate.source.clone()))
}

fn primary_is_current(evidence: &ActiveContextInput<'_>) -> bool {
    evidence
        .state
        .catalog()
        .transfer_identity(
            evidence.active.post(),
            evidence.active.identity().source().as_str(),
        )
        .as_ref()
        == Some(evidence.active.identity())
}

fn origin<'a>(
    candidate: &'a ghostr_engine::adaptive::CandidateSnapshot,
    source: &str,
) -> Option<&'a OriginHealth> {
    candidate
        .origins
        .iter()
        .find(|origin| origin.source == source)
}

fn alternate<'a>(
    evidence: &ActiveContextInput<'_>,
    candidate: &'a ghostr_engine::adaptive::CandidateSnapshot,
) -> Option<&'a OriginHealth> {
    candidate
        .origins
        .iter()
        .filter(|origin| eligible_alternate(candidate, evidence.active, origin))
        .min_by_key(|origin| {
            forecast::p95_completion_micros(evidence, candidate, &origin.source, false)
        })
}

fn eligible_alternate(
    candidate: &ghostr_engine::adaptive::CandidateSnapshot,
    active: &crate::manager::inflight::ActiveAction,
    origin: &OriginHealth,
) -> bool {
    origin.available
        && origin.source != active.identity().source().as_str()
        && !candidate
            .in_flight
            .iter()
            .any(|item| item.source == origin.source && item.request == active.request())
}

fn hedge_input(
    evidence: &ActiveContextInput<'_>,
    candidate: &ghostr_engine::adaptive::CandidateSnapshot,
    primary: &OriginHealth,
    alternate: &OriginHealth,
) -> HedgeInput {
    let request = evidence.active.request();
    let comparison = forecast::compare(evidence, candidate, primary, alternate);
    HedgeInput::new(evidence.active.action_id(), action_kind(request))
        .with_timing(elapsed_ms(evidence), comparison.trigger_ms)
        .with_value(
            comparison.loss_reduction_micros,
            comparison.duplicate_cost_micros,
        )
        .with_network_envelope(request.reserved_network_bytes())
        .with_urgency(urgent(evidence))
}

fn urgent(evidence: &ActiveContextInput<'_>) -> bool {
    evidence.active.post() == &evidence.snapshot.playback.current
        && evidence.base.mode != ControlMode::Normal
}

fn elapsed_ms(evidence: &ActiveContextInput<'_>) -> u64 {
    evidence
        .snapshot
        .observed_at_ms
        .saturating_sub(evidence.active.launched_at_ms())
}

fn action_kind(request: RetrievalRequest) -> ActionKind {
    match request {
        RetrievalRequest::FetchRange { bytes, .. } => ActionKind::FetchRange(bytes),
        RetrievalRequest::FetchWhole { contract, .. } => ActionKind::FetchWhole {
            maximum_bytes: contract.maximum_bytes(),
        },
    }
}
