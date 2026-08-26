use super::active::ActiveContextInput;
use crate::manager::hedge_tail::HedgeTailWake;
use ghostr_engine::adaptive::{
    ActionKind, ActivePlannerContext, ControlMode, HedgeInput, HedgePolicy, IdentityProof,
    OriginHealth, RetrievalRequest, SoftRequestCommitment,
};

mod forecast;
mod identity;

pub(super) struct Application {
    pub(super) context: ActivePlannerContext,
    pub(super) wake: Option<HedgeTailWake>,
    pub(super) soft: Option<SoftRequestCommitment>,
}

pub(super) fn apply(
    context: ActivePlannerContext,
    evidence: &ActiveContextInput<'_>,
) -> Application {
    let Some(candidate) = evidence
        .snapshot
        .candidates
        .iter()
        .find(|item| item.post == *evidence.active.post())
    else {
        return unavailable(context);
    };
    let Some((input, proof, alternate)) = hedge(evidence, candidate) else {
        return unavailable(context);
    };
    let eligible = eligible_at_tail(&input, proof);
    let wake = tail_wake(evidence, &input, eligible);
    let soft = eligible.then(|| hedge_commitment(evidence, alternate.clone()));
    Application {
        context: context.with_hedge(input, proof, alternate),
        wake,
        soft,
    }
}

fn tail_wake(
    evidence: &ActiveContextInput<'_>,
    input: &HedgeInput,
    eligible: bool,
) -> Option<HedgeTailWake> {
    if input.elapsed_ms >= input.tail_trigger_ms || !eligible {
        return None;
    }
    Some({
        let deadline = evidence
            .active
            .launched_at_ms()
            .saturating_add(input.tail_trigger_ms);
        HedgeTailWake::new(input.primary, deadline)
    })
}

fn eligible_at_tail(input: &HedgeInput, proof: IdentityProof) -> bool {
    let mut at_tail = input.clone();
    at_tail.elapsed_ms = at_tail.elapsed_ms.max(input.tail_trigger_ms);
    HedgePolicy::eligible(&at_tail, proof)
}

fn hedge_commitment(evidence: &ActiveContextInput<'_>, alternate: String) -> SoftRequestCommitment {
    SoftRequestCommitment::new(
        evidence.active.post().clone(),
        alternate,
        evidence.active.request(),
    )
}

fn unavailable(context: ActivePlannerContext) -> Application {
    Application {
        context,
        wake: None,
        soft: None,
    }
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
