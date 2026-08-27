use super::{AdmissionClaim, AdmissionClaimTerminal, ClaimKind};
use crate::origin_model::circuit::{RecoveryClaim, RecoveryResolution, RecoveryStage};
use crate::origin_model::exploration::ExplorationClaim;
use crate::origin_model::{OriginModel, OriginObservation, OriginOutcome, RequestMethod};

impl OriginModel {
    pub fn complete_claim(
        &mut self,
        claim: AdmissionClaim,
        terminal: AdmissionClaimTerminal<'_>,
    ) -> bool {
        match claim.0 {
            ClaimKind::Exploration(exploration) => {
                self.complete_exploration(&exploration, terminal)
            }
            ClaimKind::Recovery(recovery) => self.complete_recovery(&recovery, terminal),
        }
    }

    fn complete_exploration(
        &mut self,
        claim: &ExplorationClaim,
        terminal: AdmissionClaimTerminal<'_>,
    ) -> bool {
        match terminal {
            AdmissionClaimTerminal::NotStarted => self.exploration.release(claim),
            AdmissionClaimTerminal::Observed(item)
            | AdmissionClaimTerminal::ObservedWholeBody(item) => self.observe(item),
            AdmissionClaimTerminal::StartedWithoutObservation => {}
        }
        true
    }

    fn complete_recovery(
        &mut self,
        claim: &RecoveryClaim,
        terminal: AdmissionClaimTerminal<'_>,
    ) -> bool {
        if !self.circuits.is_current(claim) {
            if let Some((item, _)) = observed(terminal) {
                self.observe_physical(claim, item);
            }
            return false;
        }
        let Some((item, whole_body)) = observed(terminal) else {
            return self.release_recovery(claim);
        };
        if item.query.origin() != claim.key().origin {
            self.observe(item);
            self.release_recovery(claim);
            return false;
        }
        self.observe_physical(claim, item);
        let Some(resolution) = recovery_resolution(claim, item, whole_body) else {
            self.release_recovery(claim);
            return false;
        };
        self.circuits.complete(claim, resolution)
    }

    fn release_recovery(&mut self, claim: &RecoveryClaim) -> bool {
        self.circuits.complete(claim, RecoveryResolution::Released)
    }

    fn observe_physical(&mut self, claim: &RecoveryClaim, item: &OriginObservation) {
        if item.outcome == OriginOutcome::Cancelled {
            return;
        }
        self.observe_records(item);
        if super::super::circuit_key(&item.query) != *claim.key() {
            self.observe_circuit(item);
        }
    }
}

fn observed(terminal: AdmissionClaimTerminal<'_>) -> Option<(&OriginObservation, bool)> {
    match terminal {
        AdmissionClaimTerminal::Observed(item) => Some((item, false)),
        AdmissionClaimTerminal::ObservedWholeBody(item) => Some((item, true)),
        AdmissionClaimTerminal::NotStarted | AdmissionClaimTerminal::StartedWithoutObservation => {
            None
        }
    }
}

fn recovery_resolution(
    claim: &RecoveryClaim,
    item: &OriginObservation,
    whole_body: bool,
) -> Option<RecoveryResolution> {
    let physical = item.query.context.method;
    match (claim.stage(), claim.key().method, physical) {
        (RecoveryStage::Probe, RequestMethod::FullGet, RequestMethod::RangeGet) => {
            Some(sparse_resolution(item))
        }
        (RecoveryStage::Probe, RequestMethod::FullGet, RequestMethod::FullGet) => {
            Some(probe_full_resolution(item))
        }
        (RecoveryStage::Probe, RequestMethod::FullGet, _) => None,
        (_, claimed, actual) if claimed == actual => {
            Some(exact_resolution(claim, item, whole_body))
        }
        _ => None,
    }
}

fn probe_full_resolution(item: &OriginObservation) -> RecoveryResolution {
    match item.outcome {
        OriginOutcome::Success => RecoveryResolution::TrialRequired {
            at_ms: item.observed_at_ms,
        },
        OriginOutcome::Cancelled => RecoveryResolution::Released,
        OriginOutcome::Failure(_) => RecoveryResolution::Failed {
            at_ms: item.observed_at_ms,
        },
    }
}

fn exact_resolution(
    claim: &RecoveryClaim,
    item: &OriginObservation,
    whole_body: bool,
) -> RecoveryResolution {
    match item.outcome {
        OriginOutcome::Success if claim.key().method == RequestMethod::FullGet && !whole_body => {
            RecoveryResolution::Deferred {
                at_ms: item.observed_at_ms,
            }
        }
        OriginOutcome::Success if item.range_compliant != Some(false) => {
            RecoveryResolution::Recovered {
                at_ms: item.observed_at_ms,
            }
        }
        OriginOutcome::Cancelled => RecoveryResolution::Released,
        OriginOutcome::Success | OriginOutcome::Failure(_) => RecoveryResolution::Failed {
            at_ms: item.observed_at_ms,
        },
    }
}

fn sparse_resolution(item: &OriginObservation) -> RecoveryResolution {
    match item.outcome {
        OriginOutcome::Success if item.range_compliant != Some(false) => {
            RecoveryResolution::TrialRequired {
                at_ms: item.observed_at_ms,
            }
        }
        OriginOutcome::Cancelled => RecoveryResolution::Released,
        OriginOutcome::Success => RecoveryResolution::TrialRequired {
            at_ms: item.observed_at_ms,
        },
        OriginOutcome::Failure(_) => RecoveryResolution::Failed {
            at_ms: item.observed_at_ms,
        },
    }
}
