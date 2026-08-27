use crate::manager::inflight::FinishedAction;
use crate::manager::transfers::ChunkDone;
use ghostr_engine::origin_model::{AdmissionClaimTerminal, OriginModel, OriginObservation};

pub(super) fn settle(
    model: &mut OriginModel,
    done: &ChunkDone,
    finished: &mut FinishedAction,
) -> bool {
    let Some(claim) = finished.take_admission_claim() else {
        return false;
    };
    model.complete_claim(claim, terminal(done));
    true
}

fn terminal(done: &ChunkDone) -> AdmissionClaimTerminal<'_> {
    if !done.request_started {
        return AdmissionClaimTerminal::NotStarted;
    }
    let Some(observation) = done.origin.as_deref() else {
        return AdmissionClaimTerminal::StartedWithoutObservation;
    };
    observed(observation, done.whole_body_completion.is_some())
}

fn observed(observation: &OriginObservation, whole_body: bool) -> AdmissionClaimTerminal<'_> {
    if whole_body {
        AdmissionClaimTerminal::ObservedWholeBody(observation)
    } else {
        AdmissionClaimTerminal::Observed(observation)
    }
}
